import { describe, expect, test } from "bun:test";
import { appendBounded, latencyMs, percentile, summarize } from "./stats";

const samples = (values: number[]) => values.map((latencyMs, at) => ({ at, latencyMs }));

describe("latency statistics", () => {
  test("p95 reports the tail a viewer actually suffers, not the average", () => {
    const values = [...Array(19).fill(1000), 9000];
    const summary = summarize(samples(values));
    expect(summary.avg).toBe(1400);
    expect(summary.p50).toBe(1000);
    expect(summary.p95).toBe(1000);
    expect(percentile([...values].sort((a, b) => a - b), 100)).toBe(9000);
  });

  test("current latency is the most recent sample so the readout follows the stream live", () => {
    expect(summarize(samples([3000, 900])).current).toBe(900);
  });

  test("empty history yields no numbers instead of misleading zeros", () => {
    const summary = summarize([]);
    expect(summary.avg).toBeNull();
    expect(summary.p95).toBeNull();
  });

  test("history stays bounded so a long session does not grow memory", () => {
    let history = samples([]);
    for (let i = 0; i < 500; i++) history = appendBounded(history, { at: i, latencyMs: i }, 240);
    expect(history.length).toBe(240);
    expect(history[0].at).toBe(260);
  });

  test("glass-to-glass latency is wall clock minus program date time of the playhead", () => {
    expect(latencyMs(10_500, new Date(9_000))).toBe(1500);
    expect(latencyMs(10_500, null)).toBeNull();
    expect(latencyMs(10_500, new Date(20_000))).toBeNull();
  });
});
