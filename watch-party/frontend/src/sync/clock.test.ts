import { describe, expect, test } from "bun:test";
import { bestSample, pushSample, sampleFrom } from "./clock";

describe("ntp clock sync", () => {
  test("recovers a client clock that runs 5s ahead when the network is symmetric", () => {
    const skew = 5000;
    const t0 = 100_000 + skew;
    const t1 = 100_020;
    const t2 = 100_021;
    const t3 = 100_041 + skew;
    const s = sampleFrom(t0, t1, t2, t3);
    expect(s.offset).toBe(-skew);
    expect(s.rtt).toBe(40);
  });

  test("server processing time is excluded from rtt so a slow server does not look like a slow network", () => {
    const s = sampleFrom(0, 10, 510, 520);
    expect(s.rtt).toBe(20);
    expect(s.offset).toBe(0);
  });

  test("min rtt sample wins because its asymmetry error is bounded by rtt/2", () => {
    const noisy = sampleFrom(0, 300, 300, 310);
    const clean = sampleFrom(1000, 1005, 1005, 1010);
    const best = bestSample([noisy, clean]);
    expect(best).toEqual(clean);
    expect(Math.abs(best!.offset)).toBeLessThan(Math.abs(noisy.offset));
  });

  test("window forgets old samples so the estimate can follow a drifting clock", () => {
    let samples = [sampleFrom(0, 0, 0, 1)];
    for (let i = 0; i < 4; i++) samples = pushSample(samples, sampleFrom(0, 50, 50, 200), 4);
    expect(samples.length).toBe(4);
    expect(bestSample(samples)!.rtt).toBe(200);
  });

  test("no samples means no estimate", () => {
    expect(bestSample([])).toBeNull();
  });
});
