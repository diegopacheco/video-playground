import type { Level, RenditionReport } from "../types/manifest";

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function formatKbps(kbps: number): string {
  return kbps >= 1000 ? `${(kbps / 1000).toFixed(2)} Mbps` : `${Math.round(kbps)} kbps`;
}

export function levelLabel(level: Level | undefined): string {
  if (!level) return "unknown";
  return `${level.height}p (${level.width}x${level.height}, ${formatKbps(level.bitrate / 1000)})`;
}

export function switchMessage(from: Level | undefined, to: Level | undefined, at: number): string {
  const direction = from && to ? (to.bitrate > from.bitrate ? "up" : "down") : "start";
  return `${at.toFixed(1)}s ${direction} -> ${to ? `${to.height}p` : "unknown"}`;
}

export function bitsPerPixel(r: RenditionReport, fps: number): number {
  return (r.average_kbps * 1000) / (r.width * r.height * fps);
}

export function maxSegmentBytes(renditions: RenditionReport[]): number {
  return renditions.reduce((max, r) => r.segments.reduce((m, s) => Math.max(m, s.bytes), max), 0);
}

export function bufferAhead(ranges: { start: number; end: number }[], time: number): number {
  const range = ranges.find((r) => time >= r.start && time <= r.end);
  return range ? range.end - time : 0;
}

export function peakToAverage(r: RenditionReport): number {
  return r.average_kbps > 0 ? r.peak_kbps / r.average_kbps : 0;
}
