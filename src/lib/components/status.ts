// Status presentation for a metric entry. This is presentation wording only —
// labels and a marker shape derived from the four state concepts. It does NOT
// infer whether a value is good/bad/safe/fast/compatible; a normal absence is
// never dressed up as an error. The text label is the primary status cue (so
// status is never conveyed by colour or shape alone); the marker is a
// secondary, decorative cue.

import type { SnapshotEntryView } from "../types";

export type StatusTone = "available" | "unavailable" | "unsupported" | "failed";
export type StatusMarker = "filled" | "outline" | "square" | "triangle";

export interface MetricStatus {
  label: string;
  tone: StatusTone;
  marker: StatusMarker;
}

export function metricStatus(entry: SnapshotEntryView): MetricStatus {
  if (entry.support.kind === "unsupported") {
    return { label: "Not supported", tone: "unsupported", marker: "square" };
  }
  switch (entry.outcome.kind) {
    case "ok":
      return { label: "Available", tone: "available", marker: "filled" };
    case "failed":
      return {
        label: "Acquisition failed",
        tone: "failed",
        marker: "triangle",
      };
    case "no_value":
      break;
  }
  // no_value: the reason is conveyed by availability, not by alarm.
  if (entry.source_availability && entry.source_availability !== "ready") {
    return {
      label: "Source unavailable",
      tone: "unavailable",
      marker: "outline",
    };
  }
  if (entry.metric_availability === "not_exposed") {
    return { label: "Not exposed", tone: "unavailable", marker: "outline" };
  }
  if (entry.metric_availability === "transiently_unavailable") {
    return {
      label: "Temporarily unavailable",
      tone: "unavailable",
      marker: "outline",
    };
  }
  return { label: "Unavailable", tone: "unavailable", marker: "outline" };
}
