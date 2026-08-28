import { describe, expect, it } from "vitest";
import { metricStatus } from "./status";
import type { SnapshotEntryView } from "../types";

function entry(overrides: Partial<SnapshotEntryView>): SnapshotEntryView {
  return {
    display_name: "Test metric",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: "available",
    outcome: { kind: "no_value" },
    interpretation: "",
    why_it_matters: "",
    deeper: null,
    ...overrides,
  };
}

describe("metricStatus", () => {
  it("labels an available successful metric", () => {
    const s = metricStatus(
      entry({
        outcome: {
          kind: "ok",
          value: 100,
          unit: "bytes",
          provenance: "operating_system_reported",
        },
      }),
    );
    expect(s.label).toBe("Available");
    expect(s.tone).toBe("available");
    expect(s.marker).toBe("filled");
  });

  it("labels an unsupported capability distinctly, regardless of outcome", () => {
    const s = metricStatus(
      entry({
        support: { kind: "unsupported", limitation_code: "requires_driver" },
        source_availability: null,
        metric_availability: "not_applicable",
        outcome: { kind: "no_value" },
      }),
    );
    expect(s.label).toBe("Not supported");
    expect(s.tone).toBe("unsupported");
    expect(s.marker).toBe("square");
  });

  it("labels source not detected as Source unavailable (not an error)", () => {
    const s = metricStatus(
      entry({
        source_availability: "not_detected",
        metric_availability: null,
        outcome: { kind: "no_value" },
      }),
    );
    expect(s.label).toBe("Source unavailable");
    expect(s.tone).toBe("unavailable");
    // Not the failure tone — normal absence is not dressed as an error.
    expect(s.tone).not.toBe("failed");
  });

  it("labels source not running as Source unavailable", () => {
    const s = metricStatus(
      entry({
        source_availability: "not_running",
        metric_availability: null,
        outcome: { kind: "no_value" },
      }),
    );
    expect(s.label).toBe("Source unavailable");
    expect(s.tone).toBe("unavailable");
  });

  it("labels a metric not exposed (source ready) as Not exposed", () => {
    const s = metricStatus(
      entry({
        source_availability: "ready",
        metric_availability: "not_exposed",
        outcome: { kind: "no_value" },
      }),
    );
    expect(s.label).toBe("Not exposed");
    expect(s.tone).toBe("unavailable");
  });

  it("labels a transiently unavailable metric as Temporarily unavailable", () => {
    const s = metricStatus(
      entry({
        source_availability: "ready",
        metric_availability: "transiently_unavailable",
        outcome: { kind: "no_value" },
      }),
    );
    expect(s.label).toBe("Temporarily unavailable");
    expect(s.tone).toBe("unavailable");
  });

  it("labels an acquisition failure distinctly (timeout)", () => {
    const s = metricStatus(
      entry({
        outcome: { kind: "failed", category: "timeout" },
      }),
    );
    expect(s.label).toBe("Acquisition failed");
    expect(s.tone).toBe("failed");
    expect(s.marker).toBe("triangle");
  });

  it("never infers good/bad — availability and failure are separate tones", () => {
    const ok = metricStatus(
      entry({
        outcome: {
          kind: "ok",
          value: 1,
          unit: "bytes",
          provenance: "estimated",
        },
      }),
    );
    const unavailable = metricStatus(
      entry({
        outcome: { kind: "no_value" },
        metric_availability: "not_exposed",
      }),
    );
    const failed = metricStatus(
      entry({ outcome: { kind: "failed", category: "timeout" } }),
    );
    const tones = new Set([ok.tone, unavailable.tone, failed.tone]);
    // Three distinct tones for three distinct meanings.
    expect(tones.size).toBe(3);
  });

  it("every status carries a non-empty text label (never colour/shape alone)", () => {
    const cases: SnapshotEntryView[] = [
      entry({
        outcome: {
          kind: "ok",
          value: 1,
          unit: "bytes",
          provenance: "operating_system_reported",
        },
      }),
      entry({
        support: { kind: "unsupported", limitation_code: "requires_driver" },
        source_availability: null,
        metric_availability: "not_applicable",
        outcome: { kind: "no_value" },
      }),
      entry({
        source_availability: "not_detected",
        metric_availability: null,
        outcome: { kind: "no_value" },
      }),
      entry({
        source_availability: "ready",
        metric_availability: "not_exposed",
        outcome: { kind: "no_value" },
      }),
      entry({
        source_availability: "ready",
        metric_availability: "transiently_unavailable",
        outcome: { kind: "no_value" },
      }),
      entry({ outcome: { kind: "failed", category: "timeout" } }),
    ];
    for (const c of cases) {
      expect(metricStatus(c).label.length).toBeGreaterThan(0);
    }
  });
});
