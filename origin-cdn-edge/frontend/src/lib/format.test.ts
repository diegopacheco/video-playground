import { describe, expect, test } from "bun:test";
import { formatBytes, formatPercent } from "./format";

describe("formatPercent", () => {
  test("hit ratio is shown with one decimal so small gains under load are visible", () => {
    expect(formatPercent(0.8734)).toBe("87.3%");
  });

  test("never shows impossible ratios when counters are reset mid poll", () => {
    expect(formatPercent(Number.NaN)).toBe("0.0%");
    expect(formatPercent(1.4)).toBe("100.0%");
    expect(formatPercent(-0.2)).toBe("0.0%");
  });
});

describe("formatBytes", () => {
  test("cache budget is readable in the unit operators configure it in", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(8 * 1024 * 1024)).toBe("8.0 MB");
  });
});
