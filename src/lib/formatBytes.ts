// Format a reported model size for a reader.
//
// The size Ollama reports per model entry is a byte count. This helper formats
// it as a short human-readable string (e.g. `4.98 GB`). It is the presentation
// of a *reported* size, not a claim about exact on-disk usage — Ollama models
// can share blobs, so the reported size may not equal marginal disk consumed.
//
// `null`/`undefined` (absent or invalid size) returns `null` so the UI omits
// the size for that entry rather than showing `0` (which would be mistaken for
// a real zero-size model). No aggregate total is computed here.

const UNITS = ["B", "KB", "MB", "GB", "TB", "PB", "EB"] as const;

export const MAX_EXACT_PRESENTATION_INTEGER = Number.MAX_SAFE_INTEGER;

/** Whether a numeric byte value can be presented as an exact integer without
 * silently rounding it in the frontend view representation. */
export function isExactlyRepresentableByteCount(value: number): boolean {
  return Number.isSafeInteger(value) && value >= 0;
}

/** Format a byte count as a short decimal string, or `null` when the size is
 * absent/invalid. Uses base-1000 divisions (e.g. `4.98 GB`). */
export function formatBytesFriendly(
  bytes: number | null | undefined,
): string | null {
  if (bytes === null || bytes === undefined) return null;
  if (!Number.isFinite(bytes) || bytes < 0) return null;
  if (bytes < 1000) {
    const formatted = Number.isInteger(bytes) ? bytes.toString() : bytes.toFixed(2);
    return `${formatted} B`;
  }
  const tier = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1000)),
    UNITS.length - 1,
  );
  const value = bytes / Math.pow(1000, tier);
  // Two decimals below 100 of a unit; no decimals once the value is large.
  const formatted = value >= 100 ? value.toFixed(0) : value.toFixed(2);
  return `${formatted} ${UNITS[tier]}`;
}

/** Format an exact byte count with stable en-US grouping. Invalid, fractional,
 * or unsafe numeric values return `null` rather than a rounded integer. */
export function formatExactBytes(
  bytes: number | null | undefined,
): string | null {
  if (
    bytes === null ||
    bytes === undefined ||
    !isExactlyRepresentableByteCount(bytes)
  )
    return null;
  const unit = bytes === 1 ? "byte" : "bytes";
  return `${bytes.toLocaleString("en-US")} ${unit}`;
}

/** Backwards-compatible friendly formatter used by existing model surfaces. */
export function formatBytes(bytes: number | null | undefined): string | null {
  return formatBytesFriendly(bytes);
}
