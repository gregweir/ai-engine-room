import type { ReportSaveResult } from "./types";

const messages: Record<ReportSaveResult, string> = {
  saved: "Report saved as a plain-text file.",
  cancelled: "Save cancelled. No report file was created.",
  busy: "A report save is already in progress.",
  preview_changed: "The report changed before saving. Review it and try again.",
  invalid_destination: "Choose a filename ending in .txt.",
  destination_exists:
    "That file already exists. AI Engine Room did not replace it. Choose a different name.",
  unavailable:
    "Saving is not available for that location. You can still copy the report.",
  failed: "Could not save the report. No completed report file was created.",
  cleanup_incomplete:
    "Saving did not complete, and AI Engine Room could not confirm removal of its temporary file. Check the location you chose.",
  completion_uncertain:
    "The operating system did not confirm whether the report was saved. Check the location you chose before trying again.",
};

export function reportSaveMessage(result: ReportSaveResult): string {
  return messages[result];
}

export function reportSaveRole(result: ReportSaveResult): "status" | "alert" {
  return result === "saved" || result === "cancelled" ? "status" : "alert";
}
