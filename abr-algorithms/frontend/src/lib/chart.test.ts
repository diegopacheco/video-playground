import { describe, expect, test } from "bun:test";
import { bitrateSteps, bufferPoints, linearScale, niceTicks, stallIntervals } from "./chart";
import type { SegmentRecord } from "../types/abr";

function segment(partial: Partial<SegmentRecord>): SegmentRecord {
  return {
    index: 0,
    quality: 0,
    bitrate_kbps: 300,
    request_s: 0,
    buffer_start_s: 0,
    done_s: 0,
    throughput_kbps: 0,
    buffer_s: 0,
    stall_s: 0,
    idle_s: 0,
    ...partial,
  };
}

describe("buffer curve", () => {
  test("drops to zero and stays flat during a stall so rebuffering is visible", () => {
    const points = bufferPoints(
      [segment({ request_s: 10, buffer_start_s: 2, done_s: 15, buffer_s: 4, stall_s: 3 })],
      4,
    );
    expect(points).toEqual([
      { x: 0, y: 0 },
      { x: 10, y: 2 },
      { x: 12, y: 0 },
      { x: 15, y: 0 },
      { x: 15, y: 4 },
    ]);
  });

  test("drains linearly during a download that finishes in time", () => {
    const points = bufferPoints([segment({ request_s: 0, buffer_start_s: 10, done_s: 2, buffer_s: 12 })], 4);
    expect(points[2]).toEqual({ x: 2, y: 8 });
    expect(points[3]).toEqual({ x: 2, y: 12 });
  });
});

describe("bitrate steps", () => {
  test("holds each bitrate until the next request so switches are vertical", () => {
    const points = bitrateSteps([
      segment({ request_s: 0, done_s: 1, bitrate_kbps: 300 }),
      segment({ request_s: 3, done_s: 4, bitrate_kbps: 1200 }),
    ]);
    expect(points).toEqual([
      { x: 0, y: 300 },
      { x: 3, y: 300 },
      { x: 3, y: 1200 },
      { x: 4, y: 1200 },
    ]);
  });
});

test("stall intervals end when the stalled segment arrives", () => {
  expect(stallIntervals([segment({ done_s: 20, stall_s: 5 }), segment({ done_s: 24 })])).toEqual([[15, 20]]);
});

test("linear scale maps domain ends to range ends including inverted y axes", () => {
  const sy = linearScale([0, 100], [200, 0]);
  expect(sy(0)).toBe(200);
  expect(sy(100)).toBe(0);
  expect(sy(50)).toBe(100);
});

test("ticks cover the maximum with round steps", () => {
  expect(niceTicks(4300, 4)).toEqual([0, 2000, 4000]);
  expect(niceTicks(30, 3)).toEqual([0, 10, 20, 30]);
});
