import type { LineKind } from "./types";

export function classifyLine(line: string): LineKind {
  if (line.startsWith("#EXT-X-CUE") || (line.startsWith("#EXT-X-DATERANGE") && line.includes("SCTE35"))) return "cue";
  if (line === "#EXT-X-DISCONTINUITY") return "discontinuity";
  if (line.startsWith("/api/track/")) return "ad";
  if (line.startsWith("/media/ads/")) return "slate";
  if (line.startsWith("#EXTINF")) return "info";
  if (line.startsWith("#")) return "header";
  return "content";
}

export function sessionFor(user: string, now: number): string {
  return `${user}-${now.toString(36)}`;
}

export function countKind(manifest: string, kind: LineKind): number {
  return manifest.split("\n").filter((l) => l.length > 0 && classifyLine(l) === kind).length;
}
