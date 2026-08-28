import type { SnapshotEntryView, SnapshotView } from "./types";

export const AVAILABLE_MEMORY_HISTORY_LIMIT = 12;
export const AVAILABLE_MEMORY_DISPLAY_NAME = "Available memory";

export type AvailableMemoryUnavailableReason =
  | "no_value"
  | "failed"
  | "unsupported"
  | "missing"
  | "snapshot_unavailable"
  | "snapshot_rejected"
  | "not_safely_representable";

export type AvailableMemoryEvent =
  | {
      kind: "value";
      bytes: number;
      provenance: "operating_system_reported";
    }
  | {
      kind: "unavailable";
      reason: AvailableMemoryUnavailableReason;
      approximate_bytes?: number;
    };

export interface AvailableMemoryHistoryItem {
  observation_id: number;
  snapshot_at: number | null;
  event: AvailableMemoryEvent;
}

export interface AppendAvailableMemoryResult {
  history: AvailableMemoryHistoryItem[];
  nextObservationId: number;
}

function availableMemoryEntry(
  snapshot: SnapshotView,
): SnapshotEntryView | undefined {
  return snapshot.entries.find(
    (entry) => entry.display_name === AVAILABLE_MEMORY_DISPLAY_NAME,
  );
}

export function eventFromAvailableMemorySnapshot(
  snapshot: SnapshotView | null,
  nullReason:
    "snapshot_unavailable" | "snapshot_rejected" = "snapshot_unavailable",
): AvailableMemoryEvent {
  if (snapshot === null) return { kind: "unavailable", reason: nullReason };

  const entry = availableMemoryEntry(snapshot);
  if (!entry) return { kind: "unavailable", reason: "missing" };
  if (entry.support.kind === "unsupported")
    return { kind: "unavailable", reason: "unsupported" };
  if (entry.outcome.kind === "no_value")
    return { kind: "unavailable", reason: "no_value" };
  if (entry.outcome.kind === "failed")
    return { kind: "unavailable", reason: "failed" };

  const { value, unit, provenance } = entry.outcome;
  if (
    entry.unit !== "bytes" ||
    unit !== "bytes" ||
    provenance !== "operating_system_reported"
  ) {
    return { kind: "unavailable", reason: "not_safely_representable" };
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    return {
      kind: "unavailable",
      reason: "not_safely_representable",
      ...(Number.isFinite(value) && value >= 0
        ? { approximate_bytes: value }
        : {}),
    };
  }

  return { kind: "value", bytes: value, provenance };
}

export function appendAvailableMemoryObservation(
  history: readonly AvailableMemoryHistoryItem[],
  snapshot: SnapshotView | null,
  nextObservationId: number,
  nullReason:
    "snapshot_unavailable" | "snapshot_rejected" = "snapshot_unavailable",
): AppendAvailableMemoryResult {
  const next = [
    ...history,
    {
      observation_id: nextObservationId,
      snapshot_at: snapshot?.at ?? null,
      event: eventFromAvailableMemorySnapshot(snapshot, nullReason),
    },
  ].slice(-AVAILABLE_MEMORY_HISTORY_LIMIT);

  return { history: next, nextObservationId: nextObservationId + 1 };
}

export function unavailableReasonLabel(
  reason: AvailableMemoryUnavailableReason,
): string {
  switch (reason) {
    case "no_value":
      return "No value reported";
    case "failed":
      return "Acquisition failed";
    case "unsupported":
      return "Unsupported in this build";
    case "missing":
      return "Available-memory entry missing";
    case "snapshot_unavailable":
      return "Snapshot unavailable";
    case "snapshot_rejected":
      return "Snapshot acquisition unavailable";
    case "not_safely_representable":
      return "Value not safely representable";
  }
}
