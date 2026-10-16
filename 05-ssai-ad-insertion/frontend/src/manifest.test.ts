import { describe, expect, test } from "bun:test";
import { classifyLine, countKind, sessionFor } from "./manifest";

describe("classifyLine", () => {
  test("both scte35 signalling styles are highlighted as cues so operators can spot the break", () => {
    expect(classifyLine("#EXT-X-CUE-OUT:DURATION=20")).toBe("cue");
    expect(classifyLine("#EXT-X-CUE-IN")).toBe("cue");
    expect(classifyLine('#EXT-X-DATERANGE:ID="b",PLANNED-DURATION=12,SCTE35-OUT=0xFC')).toBe("cue");
    expect(classifyLine('#EXT-X-DATERANGE:ID="chapter"')).toBe("header");
  });

  test("tracked ads and untracked slate are told apart because only ads fire beacons", () => {
    expect(classifyLine("/api/track/alice/s/break-1/acme/0.ts")).toBe("ad");
    expect(classifyLine("/media/ads/slate/seg_000.ts")).toBe("slate");
    expect(classifyLine("/media/content/seg_003.ts")).toBe("content");
  });

  test("discontinuity count reveals how many timestamp resets the player must handle", () => {
    const manifest = "#EXTM3U\n#EXT-X-DISCONTINUITY\n/api/track/a/s/b/acme/0.ts\n#EXT-X-DISCONTINUITY\n/media/content/seg_1.ts\n";
    expect(countKind(manifest, "discontinuity")).toBe(2);
    expect(countKind(manifest, "ad")).toBe(1);
  });
});

test("each playback gets a fresh session so beacons from earlier plays are not mixed in", () => {
  expect(sessionFor("alice", 1)).not.toBe(sessionFor("alice", 2));
  expect(sessionFor("alice", 35)).toMatch(/^alice-[a-z0-9]+$/);
});
