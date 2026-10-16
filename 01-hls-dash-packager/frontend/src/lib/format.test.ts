import { describe, expect, test } from "bun:test";
import { bitsPerPixel, bufferAhead, formatBytes, formatKbps, levelLabel, maxSegmentBytes, peakToAverage, switchMessage } from "./format";
import type { RenditionReport } from "../types/manifest";

function rendition(id: string, width: number, height: number, avg: number, peak: number, sizes: number[]): RenditionReport {
  return {
    id,
    width,
    height,
    target_kbps: avg,
    codecs: "avc1.64001e",
    playlist: `${id}/playlist.m3u8`,
    init_bytes: 800,
    total_bytes: sizes.reduce((a, b) => a + b, 0),
    average_kbps: avg,
    peak_kbps: peak,
    keyframes: [0, 2],
    segments: sizes.map((bytes, i) => ({ index: i + 1, uri: `${id}/seg_${i}.m4s`, start: i * 2, duration: 2, bytes, kbps: (bytes * 8) / 2000 })),
  };
}

describe("format", () => {
  test("byte sizes use the unit a human reads segment sizes in", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.00 MB");
  });

  test("bitrates switch to Mbps above one megabit", () => {
    expect(formatKbps(400)).toBe("400 kbps");
    expect(formatKbps(4769.4)).toBe("4.77 Mbps");
  });

  test("level label shows resolution and bandwidth so a forced level is unambiguous", () => {
    expect(levelLabel({ index: 3, width: 1920, height: 1080, bitrate: 5_241_900 })).toBe("1080p (1920x1080, 5.24 Mbps)");
    expect(levelLabel(undefined)).toBe("unknown");
  });

  test("switch messages tell whether ABR moved up or down the ladder", () => {
    const low = { index: 0, width: 426, height: 240, bitrate: 400_000 };
    const high = { index: 3, width: 1920, height: 1080, bitrate: 5_000_000 };
    expect(switchMessage(low, high, 4)).toBe("4.0s up -> 1080p");
    expect(switchMessage(high, low, 6.25)).toBe("6.3s down -> 240p");
    expect(switchMessage(undefined, low, 0)).toBe("0.0s start -> 240p");
  });

  test("bits per pixel drops as resolution climbs, which is why higher rungs need less than linear bitrate", () => {
    const low = rendition("240p", 426, 240, 400, 440, [100_000]);
    const high = rendition("1080p", 1920, 1080, 5000, 5500, [1_250_000]);
    expect(bitsPerPixel(low, 24)).toBeGreaterThan(bitsPerPixel(high, 24));
  });

  test("chart scale uses the single largest segment across the ladder", () => {
    expect(maxSegmentBytes([rendition("a", 2, 2, 1, 1, [10, 30]), rendition("b", 2, 2, 1, 1, [25])])).toBe(30);
    expect(maxSegmentBytes([])).toBe(0);
  });

  test("buffer ahead counts only the range holding the playhead, since a gap stalls playback", () => {
    const ranges = [
      { start: 0, end: 6 },
      { start: 8, end: 20 },
    ];
    expect(bufferAhead(ranges, 4)).toBe(2);
    expect(bufferAhead(ranges, 7)).toBe(0);
    expect(bufferAhead([], 1)).toBe(0);
  });

  test("peak to average ratio exposes how bursty the VBV-capped encode is", () => {
    expect(peakToAverage(rendition("a", 2, 2, 1000, 1200, [1]))).toBeCloseTo(1.2);
    expect(peakToAverage(rendition("a", 2, 2, 0, 1200, [1]))).toBe(0);
  });
});
