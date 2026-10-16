import { describe, expect, test } from "bun:test";
import { countStatuses, pushBounded, windowHitRatio } from "./history";

describe("pushBounded", () => {
  test("keeps only the newest samples so the chart memory stays flat during long runs", () => {
    let history: number[] = [];
    for (let i = 0; i < 100; i++) history = pushBounded(history, i, 60);
    expect(history.length).toBe(60);
    expect(history[0]).toBe(40);
    expect(history[59]).toBe(99);
  });
});

describe("windowHitRatio", () => {
  test("measures the last interval, not the lifetime ratio, so warmup does not hide a regression", () => {
    const previous = { requests: 1000, hits: 100, stale: 0 };
    const current = { requests: 1100, hits: 190, stale: 5 };
    expect(windowHitRatio(previous, current)).toBeCloseTo(0.95);
  });

  test("returns null after a counter reset instead of a negative ratio", () => {
    expect(windowHitRatio({ requests: 50, hits: 40, stale: 0 }, { requests: 3, hits: 1, stale: 0 })).toBeNull();
  });

  test("returns null when there was no traffic in the interval", () => {
    const sample = { requests: 10, hits: 5, stale: 0 };
    expect(windowHitRatio(sample, sample)).toBeNull();
  });
});

describe("countStatuses", () => {
  test("groups a probe burst by X-Cache so coalesced misses are visible at a glance", () => {
    const counts = countStatuses([
      { status: "MISS", coalesced: false, millis: 80 },
      { status: "MISS", coalesced: true, millis: 81 },
      { status: "HIT", coalesced: false, millis: 1 },
    ]);
    expect(counts).toEqual({ HIT: 1, MISS: 2, STALE: 0, ERROR: 0 });
  });
});
