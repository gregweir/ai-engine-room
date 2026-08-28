import { describe, expect, it } from "vitest";
import type { SnapshotEntryView, SnapshotView } from "./types";
import {
  appendAvailableMemoryObservation,
  AVAILABLE_MEMORY_HISTORY_LIMIT,
  eventFromAvailableMemorySnapshot,
  unavailableReasonLabel,
} from "./availableMemoryHistory";

function entry(
  value: number | "no_value" | "failed" = 4096,
): SnapshotEntryView {
  return {
    display_name: "Available memory",
    unit: "bytes",
    support: { kind: "supported" },
    source_availability: "ready",
    metric_availability: value === "no_value" ? "not_exposed" : "available",
    outcome:
      value === "no_value"
        ? { kind: "no_value" }
        : value === "failed"
          ? { kind: "failed", category: "controlled" }
          : {
              kind: "ok",
              value,
              unit: "bytes",
              provenance: "operating_system_reported",
            },
    interpretation: "Controlled artificial interpretation.",
    why_it_matters: "Controlled artificial context.",
    deeper: null,
  };
}

function snapshot(
  value: number | "no_value" | "failed" = 4096,
  at = 1,
): SnapshotView {
  return { at, entries: [entry(value)] };
}

describe("Available-memory session history", () => {
  it("extracts exact artificial bytes, including successful zero", () => {
    expect(eventFromAvailableMemorySnapshot(snapshot(12_961_325_056))).toEqual({
      kind: "value",
      bytes: 12_961_325_056,
      provenance: "operating_system_reported",
    });
    expect(eventFromAvailableMemorySnapshot(snapshot(0))).toEqual({
      kind: "value",
      bytes: 0,
      provenance: "operating_system_reported",
    });
  });

  it("maps every controlled nonnumeric category without retaining raw errors", () => {
    expect(eventFromAvailableMemorySnapshot(snapshot("no_value"))).toEqual({
      kind: "unavailable",
      reason: "no_value",
    });
    expect(eventFromAvailableMemorySnapshot(snapshot("failed"))).toEqual({
      kind: "unavailable",
      reason: "failed",
    });
    const unsupported = snapshot();
    unsupported.entries[0]!.support = {
      kind: "unsupported",
      limitation_code: "private raw reason is not retained",
    };
    expect(eventFromAvailableMemorySnapshot(unsupported)).toEqual({
      kind: "unavailable",
      reason: "unsupported",
    });
    expect(eventFromAvailableMemorySnapshot({ at: 2, entries: [] })).toEqual({
      kind: "unavailable",
      reason: "missing",
    });
    expect(eventFromAvailableMemorySnapshot(null)).toEqual({
      kind: "unavailable",
      reason: "snapshot_unavailable",
    });
    expect(eventFromAvailableMemorySnapshot(null, "snapshot_rejected")).toEqual(
      { kind: "unavailable", reason: "snapshot_rejected" },
    );
  });

  it("retains a safe approximate display value without making it a numeric point", () => {
    expect(
      eventFromAvailableMemorySnapshot(snapshot(Number.MAX_SAFE_INTEGER + 1)),
    ).toEqual({
      kind: "unavailable",
      reason: "not_safely_representable",
      approximate_bytes: Number.MAX_SAFE_INTEGER + 1,
    });
    expect(eventFromAvailableMemorySnapshot(snapshot(1.5))).toEqual({
      kind: "unavailable",
      reason: "not_safely_representable",
      approximate_bytes: 1.5,
    });
  });

  it.each([-1, Number.NaN, Number.POSITIVE_INFINITY])(
    "treats invalid value %s as a nonnumeric event without an approximation",
    (value) => {
      expect(eventFromAvailableMemorySnapshot(snapshot(value))).toEqual({
        kind: "unavailable",
        reason: "not_safely_representable",
      });
    },
  );

  it("rejects a wrong unit or provenance", () => {
    const wrongUnit = snapshot();
    wrongUnit.entries[0]!.unit = "count";
    expect(eventFromAvailableMemorySnapshot(wrongUnit).kind).toBe(
      "unavailable",
    );
    const wrongProvenance = snapshot();
    if (wrongProvenance.entries[0]!.outcome.kind === "ok")
      wrongProvenance.entries[0]!.outcome.provenance = "runtime_reported";
    expect(eventFromAvailableMemorySnapshot(wrongProvenance).kind).toBe(
      "unavailable",
    );
  });

  it("appends oldest-to-newest with monotonic ids, timestamps, and duplicates", () => {
    const first = appendAvailableMemoryObservation([], snapshot(7, 10), 1);
    const second = appendAvailableMemoryObservation(
      first.history,
      snapshot(7, 20),
      2,
    );
    expect(second.nextObservationId).toBe(3);
    expect(second.history.map((item) => item.observation_id)).toEqual([1, 2]);
    expect(second.history.map((item) => item.snapshot_at)).toEqual([10, 20]);
    expect(second.history.map((item) => item.event)).toEqual([
      { kind: "value", bytes: 7, provenance: "operating_system_reported" },
      { kind: "value", bytes: 7, provenance: "operating_system_reported" },
    ]);
    expect(first.history).toHaveLength(1);
  });

  it("retains only the newest 12 events without renumbering them", () => {
    let history = [] as ReturnType<
      typeof appendAvailableMemoryObservation
    >["history"];
    let nextObservationId = 1;
    for (let value = 0; value < AVAILABLE_MEMORY_HISTORY_LIMIT + 3; value++) {
      ({ history, nextObservationId } = appendAvailableMemoryObservation(
        history,
        snapshot(value, value),
        nextObservationId,
      ));
    }
    expect(history).toHaveLength(12);
    expect(history[0]!.observation_id).toBe(4);
    expect(history.at(-1)!.observation_id).toBe(15);
    expect(nextObservationId).toBe(16);
  });

  it("provides only fixed reader labels for gap reasons", () => {
    expect(unavailableReasonLabel("failed")).toBe("Acquisition failed");
    expect(unavailableReasonLabel("snapshot_rejected")).not.toMatch(
      /error|exception|stack|path/i,
    );
  });
});
