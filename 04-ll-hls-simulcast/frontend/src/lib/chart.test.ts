import { describe, expect, test } from "bun:test";
import { buildChart, niceMax } from "./chart";

const geometry = { width: 600, height: 200, windowMs: 60_000 };

describe("latency chart", () => {
  test("y axis always fits the worst visible latency so spikes are never clipped off-scale", () => {
    expect(niceMax(0)).toBe(1000);
    expect(niceMax(1240)).toBe(2000);
    expect(niceMax(6100)).toBe(8000);
    expect(niceMax(6100) / 4 % 500).toBe(0);
  });

  test("both modes share one y scale so the low-latency gain is visually comparable", () => {
    const model = buildChart(
      { ll: [{ at: 100_000, latencyMs: 1000 }], regular: [{ at: 100_000, latencyMs: 6000 }] },
      100_000,
      geometry,
    );
    expect(model.yMax).toBe(6000);
    expect(model.paths.regular).toBe("M600.0,0.0");
    expect(model.paths.ll).toBe("M600.0,166.7");
  });

  test("samples older than the window are not drawn", () => {
    const model = buildChart(
      { ll: [{ at: 1_000, latencyMs: 9000 }, { at: 90_000, latencyMs: 800 }], regular: [] },
      100_000,
      geometry,
    );
    expect(model.yMax).toBe(1000);
    expect(model.paths.ll.startsWith("M")).toBe(true);
    expect(model.paths.ll.split(" ").length).toBe(1);
  });
});
