import { describe, expect, test } from "bun:test";
import { correct, driftMs, expectedPosition, MAX_NUDGE } from "./drift";
import type { PlaybackState } from "../types/protocol";

const playing: PlaybackState = { playing: true, position: 10, rate: 1, at: 1_000 };

describe("expected position", () => {
  test("advances by elapsed server time times rate", () => {
    expect(expectedPosition(playing, 3_500)).toBeCloseTo(12.5, 9);
    expect(expectedPosition({ ...playing, rate: 2 }, 2_000)).toBeCloseTo(12, 9);
  });

  test("paused state stays put no matter how much time passes", () => {
    expect(expectedPosition({ ...playing, playing: false }, 999_999)).toBe(10);
  });

  test("never rewinds when the follower clock estimate is slightly behind the state time", () => {
    expect(expectedPosition(playing, 900)).toBe(10);
  });
});

describe("drift correction", () => {
  test("tiny drift is left alone so the player is not constantly retimed", () => {
    expect(correct(25, playing, 10)).toEqual({ kind: "hold", rate: 1 });
  });

  test("follower ahead slows down and follower behind speeds up", () => {
    const ahead = correct(80, playing, 10);
    const behind = correct(-80, playing, 10);
    expect(ahead.kind).toBe("nudge");
    expect(ahead.rate).toBeLessThan(1);
    expect(behind.rate).toBeGreaterThan(1);
  });

  test("an active nudge keeps going until drift settles near zero so the follower does not park at the deadband edge", () => {
    expect(correct(30, playing, 10, true).kind).toBe("nudge");
    expect(correct(30, playing, 10, false).kind).toBe("hold");
    expect(correct(8, playing, 10, true).kind).toBe("hold");
  });

  test("rate nudge is capped at 5% so audio pitch change stays unnoticeable", () => {
    expect(correct(900, playing, 10).rate).toBeCloseTo(1 - MAX_NUDGE, 9);
    expect(correct(-900, playing, 10).rate).toBeCloseTo(1 + MAX_NUDGE, 9);
  });

  test("nudge is relative to the party rate so a 2x party stays at 2x", () => {
    const c = correct(80, { ...playing, rate: 2 }, 10);
    expect(c.rate).toBeLessThan(2);
    expect(c.rate).toBeGreaterThan(1.9);
  });

  test("large drift seeks because a 5% nudge would take too long to close it", () => {
    expect(correct(-3_000, playing, 42)).toEqual({ kind: "seek", rate: 1, position: 42 });
  });

  test("paused followers snap to the paused frame", () => {
    const paused = { ...playing, playing: false };
    expect(correct(200, paused, 10)).toEqual({ kind: "seek", rate: 1, position: 10 });
    expect(correct(10, paused, 10).kind).toBe("hold");
  });

  test("drift is actual minus expected in milliseconds", () => {
    expect(driftMs(10.25, 10)).toBeCloseTo(250, 9);
  });
});
