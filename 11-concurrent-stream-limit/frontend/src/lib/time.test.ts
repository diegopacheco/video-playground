import { describe, expect, test } from "bun:test";
import { fraction, heartbeatInterval, remainingMs, seconds } from "./time";

describe("countdown uses server time so a skewed browser clock does not lie", () => {
  test("remaining is relative to server now, not the local clock", () => {
    const localFetchedAt = 999_000_000;
    expect(remainingMs(15_000, 5_000, localFetchedAt, localFetchedAt)).toBe(10_000);
    expect(remainingMs(15_000, 5_000, localFetchedAt, localFetchedAt + 4_000)).toBe(6_000);
  });

  test("an expired lease never shows a negative countdown", () => {
    expect(remainingMs(15_000, 20_000, 0, 0)).toBe(0);
  });

  test("progress fraction is clamped to the bar", () => {
    expect(fraction(5_000, 10_000)).toBe(0.5);
    expect(fraction(20_000, 10_000)).toBe(1);
    expect(fraction(1, 0)).toBe(0);
  });

  test("seconds formatting keeps one decimal", () => {
    expect(seconds(2_345)).toBe("2.3s");
  });
});

describe("heartbeat cadence", () => {
  test("a device beats three times per ttl so one lost beat does not drop the stream", () => {
    expect(heartbeatInterval(9_000)).toBe(3_000);
    expect(heartbeatInterval(9_000) * 2).toBeLessThan(9_000);
  });

  test("tiny ttls do not cause a heartbeat storm", () => {
    expect(heartbeatInterval(100)).toBe(250);
  });
});
