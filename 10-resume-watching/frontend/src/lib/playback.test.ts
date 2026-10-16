import { describe, expect, test } from "bun:test";
import { advance, findTitle, formatTime, heartbeatDue, isFinished } from "./playback";

describe("playback", () => {
  test("position never runs past the end so heartbeats stay valid for the backend", () => {
    expect(advance(595, 10, 600)).toBe(600);
  });

  test("seeking back never produces a negative position", () => {
    expect(advance(5, -30, 600)).toBe(0);
  });

  test("finished threshold matches the backend rule of strictly above 90 percent", () => {
    expect(isFinished(540, 600)).toBe(false);
    expect(isFinished(541, 600)).toBe(true);
  });

  test("heartbeat fires only on its interval so the UI sends the cadence a real player would", () => {
    expect(heartbeatDue(10, 5)).toBe(true);
    expect(heartbeatDue(11, 5)).toBe(false);
  });

  test("unknown title falls back instead of crashing the device panel", () => {
    expect(findTitle("missing").id).toBe("dark-harbor");
  });

  test("time is shown as minutes and zero padded seconds", () => {
    expect(formatTime(125)).toBe("2:05");
  });
});
