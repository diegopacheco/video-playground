import { describe, expect, test } from "bun:test";
import { domainFor, linePath, niceMax, ratePerSecond, scaleY, yTicks } from "./chart";
import { compact, millis, percent } from "./format";
import type { Bucket } from "../types/sim";

const frame = { width: 200, height: 100, pad: 10 };

function bucket(tMs: number, origin: number): Bucket {
  return {
    t_ms: tMs,
    api_requests: 0,
    api_rejected: 0,
    origin_requests: origin,
    origin_rejected: 0,
    first_frames: 0,
    admitted: 0,
    retries: 0,
  };
}

describe("chart scaling", () => {
  test("axis max rounds up so the tallest spike is never clipped", () => {
    expect(niceMax(27_253)).toBe(50_000);
    expect(niceMax(2_000)).toBe(2_000);
    expect(niceMax(0)).toBe(1);
  });

  test("zero load sits on the baseline and peak load touches the top", () => {
    const domain = { maxX: 10, maxY: 100, log: false };
    expect(scaleY(0, domain, frame)).toBe(90);
    expect(scaleY(100, domain, frame)).toBe(10);
  });

  test("log scale keeps a 20k waiting room visible next to a 4.8M naive spike", () => {
    const domain = { maxX: 10, maxY: 5_000_000, log: true };
    const linearHeight = 90 - scaleY(20_000, { ...domain, log: false }, frame);
    const logHeight = 90 - scaleY(20_000, domain, frame);
    expect(linearHeight).toBeLessThan(1);
    expect(logHeight).toBeGreaterThan(40);
  });

  test("time window hides seconds beyond it and the y axis follows only visible data", () => {
    const line = { series: [bucket(0, 5), bucket(1000, 7), bucket(30_000, 900)], bucketMs: 1000 };
    const domain = domainFor([line], "origin_requests", 10, false);
    expect(domain.maxX).toBe(10);
    expect(domain.maxY).toBe(10);
    expect(linePath(line, "origin_requests", domain, frame).split(" ")).toHaveLength(2);
  });

  test("100 ms buckets are plotted as requests per second so capacity lines stay comparable", () => {
    expect(ratePerSecond(bucket(0, 250), "origin_requests", 100)).toBe(2500);
    const domain = domainFor([{ series: [bucket(0, 250)], bucketMs: 100 }], "origin_requests", null, false);
    expect(domain.maxY).toBe(5000);
  });

  test("log ticks are powers of ten", () => {
    expect(yTicks({ maxX: 1, maxY: 1000, log: true })).toEqual([0, 1, 10, 100, 1000]);
  });
});

describe("formatting", () => {
  test("numbers in the table stay short and honest", () => {
    expect(compact(4_812_601)).toBe("4.8M");
    expect(compact(27_253)).toBe("27k");
    expect(percent(0.0589)).toBe("5.89%");
    expect(millis(null)).toBe("-");
    expect(millis(47_619)).toBe("47.6 s");
  });
});
