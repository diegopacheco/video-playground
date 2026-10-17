import { expect, test } from "bun:test";
import { COLUMNS, bestRuns } from "./compare";
import type { AlgorithmRun, Metrics } from "../types/abr";

function run(algorithm: string, metrics: Partial<Metrics>): AlgorithmRun {
  return {
    algorithm,
    name: algorithm,
    session: { segments: [], startup_delay_s: 0, rebuffer_s: 0, rebuffer_events: 0, content_s: 0, end_s: 0 },
    metrics: {
      avg_bitrate_kbps: 0,
      switch_count: 0,
      rebuffer_s: 0,
      rebuffer_events: 0,
      rebuffer_ratio: 0,
      startup_delay_s: 0,
      qoe: 0,
      ...metrics,
    },
  };
}

function column(key: keyof Metrics) {
  const found = COLUMNS.find((c) => c.key === key);
  if (!found) throw new Error(`missing column ${key}`);
  return found;
}

test("higher QoE wins because QoE is a reward", () => {
  const runs = [run("bola", { qoe: 2.1 }), run("throughput", { qoe: 2.5 })];
  expect([...bestRuns(runs, column("qoe"))]).toEqual(["throughput"]);
});

test("less rebuffering wins because stalls hurt viewers", () => {
  const runs = [run("bola", { rebuffer_s: 4 }), run("throughput", { rebuffer_s: 1 })];
  expect([...bestRuns(runs, column("rebuffer_s"))]).toEqual(["throughput"]);
});

test("a column where everyone ties highlights nobody", () => {
  const runs = [run("bola", { startup_delay_s: 0.4 }), run("dynamic", { startup_delay_s: 0.4 })];
  expect(bestRuns(runs, column("startup_delay_s")).size).toBe(0);
});

test("a single run has nothing to be compared against", () => {
  expect(bestRuns([run("bola", { qoe: 1 })], column("qoe")).size).toBe(0);
});
