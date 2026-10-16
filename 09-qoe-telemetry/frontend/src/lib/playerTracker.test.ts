import { describe, expect, test } from "bun:test";
import { PlayerTracker } from "./playerTracker";

const tracker = () => new PlayerTracker("p1", "local-clip", "local-origin");

describe("PlayerTracker", () => {
  test("startup time is measured from the play request, and only once per session", () => {
    const t = tracker();
    t.requestPlay(1000);
    expect(t.playing(1850)).toEqual([
      { session_id: "p1", title: "local-clip", cdn: "local-origin", ts: 1850, kind: "startup", startup_ms: 850 },
    ]);
    expect(t.playing(5000)).toEqual([]);
  });

  test("initial buffering before first frame is startup, not a rebuffer", () => {
    const t = tracker();
    t.requestPlay(0);
    expect(t.waiting(100)).toEqual([]);
  });

  test("a stall emits start and end with its duration so the aggregator can compute rebuffer ratio", () => {
    const t = tracker();
    t.requestPlay(0);
    t.playing(500);
    expect(t.waiting(2000)[0]?.kind).toBe("rebuffer_start");
    expect(t.waiting(2100)).toEqual([]);
    const end = t.playing(2600)[0];
    expect(end).toMatchObject({ kind: "rebuffer_end", rebuffer_ms: 600 });
  });

  test("heartbeat played time excludes stalled time so rebuffer is not counted as watching", () => {
    const t = tracker();
    t.requestPlay(0);
    t.playing(0);
    t.waiting(1000);
    t.playing(1400);
    expect(t.heartbeat(2000, false)).toMatchObject([{ kind: "heartbeat", played_ms: 1600 }]);
  });

  test("a stall spanning a heartbeat is split, never subtracted twice", () => {
    const t = tracker();
    t.requestPlay(0);
    t.playing(0);
    t.waiting(1500);
    expect(t.heartbeat(2000, false)).toMatchObject([{ played_ms: 1500 }]);
    t.playing(2300);
    expect(t.heartbeat(4000, false)).toMatchObject([{ played_ms: 1700 }]);
  });

  test("paused time does not count as played time", () => {
    const t = tracker();
    t.requestPlay(0);
    t.playing(0);
    expect(t.heartbeat(2000, true)).toEqual([]);
  });

  test("first level load is not a switch, later changes are", () => {
    const t = tracker();
    expect(t.levelSwitched(0, 800)).toEqual([]);
    expect(t.levelSwitched(10, 800)).toEqual([]);
    expect(t.levelSwitched(20, 1800)).toMatchObject([{ kind: "bitrate_switch", from_kbps: 800, to_kbps: 1800 }]);
  });
});
