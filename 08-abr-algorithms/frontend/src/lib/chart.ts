import type { Point, SegmentRecord } from "../types/abr";

export type Scale = (value: number) => number;

export function linearScale(domain: [number, number], range: [number, number]): Scale {
  const [d0, d1] = domain;
  const [r0, r1] = range;
  const span = d1 - d0 || 1;
  return (value) => r0 + ((value - d0) / span) * (r1 - r0);
}

export function toPath(points: Point[], sx: Scale, sy: Scale): string {
  return points
    .map((p, i) => `${i === 0 ? "M" : "L"}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`)
    .join(" ");
}

export function bitrateSteps(segments: SegmentRecord[]): Point[] {
  return segments.flatMap((segment, i) => {
    const next = segments[i + 1];
    const end = next ? next.request_s : segment.done_s;
    return [
      { x: segment.request_s, y: segment.bitrate_kbps },
      { x: end, y: segment.bitrate_kbps },
    ];
  });
}

export function bufferPoints(segments: SegmentRecord[], segmentS: number): Point[] {
  const points: Point[] = [{ x: 0, y: 0 }];
  for (const s of segments) {
    points.push({ x: s.request_s, y: s.buffer_start_s });
    if (s.stall_s > 0) {
      points.push({ x: s.request_s + s.buffer_start_s, y: 0 });
      points.push({ x: s.done_s, y: 0 });
    } else {
      points.push({ x: s.done_s, y: s.buffer_s - segmentS });
    }
    points.push({ x: s.done_s, y: s.buffer_s });
  }
  return points;
}

export function bandwidthSteps(bandwidthKbps: number[], untilS: number): Point[] {
  const points: Point[] = [];
  const seconds = Math.ceil(untilS);
  for (let t = 0; t < seconds; t++) {
    const value = bandwidthKbps[t % bandwidthKbps.length] ?? 0;
    points.push({ x: t, y: value }, { x: t + 1, y: value });
  }
  return points;
}

export function stallIntervals(segments: SegmentRecord[]): Array<[number, number]> {
  return segments
    .filter((s) => s.stall_s > 0)
    .map((s) => [s.done_s - s.stall_s, s.done_s] as [number, number]);
}

export function niceTicks(max: number, count: number): number[] {
  if (max <= 0) return [0];
  const raw = max / count;
  const magnitude = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = 0; v <= max + step * 1e-9; v += step) ticks.push(Number(v.toFixed(6)));
  return ticks;
}
