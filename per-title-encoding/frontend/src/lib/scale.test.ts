import { describe, expect, test } from "bun:test";
import { formatKbps, formatPct, linearScale, linearTicks, logScale, logTicks } from "./scale";
import { ladderTotals, percentChange, resolutions } from "./ladder";
import type { Point, TitleReport } from "../types/analysis";

const point = (height: number, kbps: number, vmaf: number): Point => ({
  resolution: { width: Math.round((height * 16) / 9), height },
  target_kbps: kbps,
  bitrate_kbps: kbps,
  scores: { vmaf },
});

describe("scales", () => {
  test("log scale gives every decade the same width so 30 kbps and 4 Mbps both stay readable", () => {
    const x = logScale([10, 1000], [0, 200]);
    expect(x(10)).toBeCloseTo(0);
    expect(x(100)).toBeCloseTo(100);
    expect(x(1000)).toBeCloseTo(200);
  });

  test("linear scale maps the quality axis bottom-up", () => {
    const y = linearScale([0, 100], [300, 0]);
    expect(y(0)).toBe(300);
    expect(y(100)).toBe(0);
  });

  test("log ticks use 1-2-5 steps inside the bitrate domain only", () => {
    expect(logTicks(30, 4000)).toEqual([50, 100, 200, 500, 1000, 2000]);
  });

  test("linear ticks land on round numbers", () => {
    expect(linearTicks(0, 100, 5)).toEqual([0, 20, 40, 60, 80, 100]);
  });

  test("bitrates switch to Mbps above 1000 kbps", () => {
    expect(formatKbps(650)).toBe("650 kbps");
    expect(formatKbps(2600)).toBe("2.6 Mbps");
  });

  test("a rung already on the hull reads as 0.0% saving, never a confusing -0.0%", () => {
    expect(formatPct(-0.00001)).toBe("0.0%");
    expect(formatPct(92.64)).toBe("92.6%");
  });
});

describe("ladder helpers", () => {
  const title: TitleReport = {
    id: "t",
    name: "t",
    description: "",
    points: [point(360, 100, 99), point(180, 250, 80), point(270, 650, 96)],
    hulls: {},
    ladder: [
      { target: 60, reached: true, point: point(270, 60, 78) },
      { target: 85, reached: true, point: point(360, 100, 99) },
    ],
    generic: [
      { point: point(180, 250, 80), equal_quality_kbps: 68, saving_pct: 72.8 },
      { point: point(270, 650, 96), equal_quality_kbps: 96, saving_pct: 85.2 },
    ],
    savings: { generic_kbps: 900, per_title_kbps: 164, saving_pct: 81.8, compared_rungs: 2 },
  };

  test("totals compare the whole ladder cost, which is what storage and CDN pay for", () => {
    const totals = ladderTotals(title, "vmaf");
    expect(totals.perTitleKbps).toBe(160);
    expect(totals.genericKbps).toBe(900);
    expect(totals.perTitleTop).toBe(99);
    expect(percentChange(totals.genericKbps, totals.perTitleKbps)).toBeCloseTo(-82.2, 1);
  });

  test("resolutions are listed once each, smallest first", () => {
    expect(resolutions(title.points).map((r) => r.height)).toEqual([180, 270, 360]);
  });
});
