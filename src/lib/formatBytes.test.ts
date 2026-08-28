import { describe, expect, it } from "vitest";
import {
  formatBytes,
  formatBytesFriendly,
  formatExactBytes,
  isExactlyRepresentableByteCount,
} from "./formatBytes";

describe("formatBytes", () => {
  it("formats a multi-gigabyte reported size with GB", () => {
    // The plan's example: 4_975_683_088 bytes contains "GB".
    const out = formatBytes(4_975_683_088);
    expect(out).not.toBeNull();
    expect(out!).toContain("GB");
  });

  it("formats a 4 GB round-number size", () => {
    expect(formatBytes(4_000_000_000)).toBe("4.00 GB");
  });

  it("formats megabyte and kilobyte tiers", () => {
    expect(formatBytes(500_000_000)).toBe("500 MB");
    expect(formatBytes(2_000)).toBe("2.00 KB");
  });

  it("formats bytes below the kilobyte tier without a unit prefix", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1)).toBe("1 B");
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(999)).toBe("999 B");
    expect(formatBytes(1.5)).toBe("1.50 B");
  });

  it("returns null for an absent or invalid size so the UI omits it", () => {
    expect(formatBytes(null)).toBeNull();
    expect(formatBytes(undefined)).toBeNull();
    // Non-finite and negative are invalid: the UI omits rather than showing 0.
    expect(formatBytes(Number.NaN)).toBeNull();
    expect(formatBytes(-1)).toBeNull();
  });

  it("extends bounded large values through PB and EB", () => {
    expect(formatBytes(5_000_000_000_000)).toBe("5.00 TB");
    expect(formatBytes(5_000_000_000_000_000)).toBe("5.00 PB");
    expect(formatBytes(Number.MAX_VALUE)).toContain("EB");
  });

  it.each([
    [0, "0 B", "0 bytes"],
    [1, "1 B", "1 byte"],
    [1024, "1.02 KB", "1,024 bytes"],
    [1_048_576, "1.05 MB", "1,048,576 bytes"],
    [12_961_325_056, "12.96 GB", "12,961,325,056 bytes"],
    [13_175_230_464, "13.18 GB", "13,175,230,464 bytes"],
  ])(
    "formats %d with friendly and exact byte forms",
    (value, friendly, exact) => {
      expect(formatBytesFriendly(value)).toBe(friendly);
      expect(formatExactBytes(value)).toBe(exact);
    },
  );

  it("does not claim unsafe, fractional, or invalid values are exact", () => {
    expect(isExactlyRepresentableByteCount(Number.MAX_SAFE_INTEGER)).toBe(true);
    expect(formatExactBytes(Number.MAX_SAFE_INTEGER)).not.toBeNull();
    for (const value of [
      Number.MAX_SAFE_INTEGER + 1,
      1.5,
      -1,
      Number.NaN,
      Number.POSITIVE_INFINITY,
    ]) {
      expect(isExactlyRepresentableByteCount(value)).toBe(false);
      expect(formatExactBytes(value)).toBeNull();
    }
  });
});
