import { describe, expect, test } from "bun:test";
import { cell, health, latestWindow, mergeWindows } from "./metrics";
import type { WindowResult } from "../types/telemetry";

const row = (start: number, key: string, rebuffer = 0): WindowResult => ({
  window_start: start,
  window_end: start + 10_000,
  dimension: "cdn",
  key,
  metrics: {
    sessions: 1,
    events: 1,
    startups: 0,
    avg_startup_ms: 0,
    p95_startup_ms: 0,
    rebuffer_ratio: rebuffer,
    switches_per_min: 0,
    error_rate: 0,
    played_ms: 0,
    rebuffer_ms: 0,
  },
});

describe("metrics", () => {
  test("history reload plus live stream does not duplicate a window", () => {
    const merged = mergeWindows([row(0, "a")], [row(0, "a", 0.2), row(10_000, "a")], 10);
    expect(merged.length).toBe(2);
    expect(merged[0]?.metrics.rebuffer_ratio).toBe(0.2);
  });

  test("only the newest N windows are kept so the dashboard memory stays bounded", () => {
    const rows = [0, 1, 2, 3, 4].map((i) => row(i * 10_000, "a"));
    expect(mergeWindows([], rows, 2).map((r) => r.window_start)).toEqual([30_000, 40_000]);
  });

  test("latest window shows every cdn of the newest closed window only", () => {
    const rows = [row(0, "a"), row(10_000, "b"), row(10_000, "a")];
    expect(latestWindow(rows, "cdn").map((r) => r.key)).toEqual(["a", "b"]);
  });

  test("a window with no session starts shows no startup value instead of a misleading 0 ms", () => {
    const empty = row(0, "a").metrics;
    expect(cell("avg_startup_ms", empty)).toEqual({ text: "-", tone: null });
    expect(cell("avg_startup_ms", { ...empty, startups: 2, avg_startup_ms: 5000 })).toEqual({ text: "5000 ms", tone: "bad" });
  });

  test("a 12% rebuffer ratio is flagged bad, 0.2% is good", () => {
    expect(health("rebuffer_ratio", 0.12)).toBe("bad");
    expect(health("rebuffer_ratio", 0.002)).toBe("good");
    expect(health("sessions", 999)).toBeNull();
  });
});
