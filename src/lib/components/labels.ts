// Presentation labels for provenance and units. These are wording choices
// (labels), not technical interpretations: they describe how a value was
// obtained, never whether it is good or bad. The value is rendered plainly,
// without colour-coding or judgement.

import type { Provenance, Unit } from "../types";

export function provenanceLabel(p: Provenance): string {
  switch (p) {
    case "operating_system_reported":
      return "Reported by the operating system";
    case "driver_reported":
      return "Reported by a hardware driver";
    case "runtime_reported":
      return "Reported by the AI runtime";
    case "application_measured":
      return "Measured by Engine Room";
    case "calculated":
      return "Calculated from other metrics";
    case "estimated":
      return "Estimated from other metrics";
  }
}

export function unitLabel(u: Unit): string {
  switch (u) {
    case "bytes":
      return "bytes";
    case "count":
      return "count";
    case "percent":
      return "percent";
    case "seconds":
      return "seconds";
  }
}

export function formatValue(v: number): string {
  // Plain number, no alarmist colour or judgement. Keep integers tidy.
  return Number.isInteger(v) ? v.toLocaleString("en-US") : v.toString();
}
