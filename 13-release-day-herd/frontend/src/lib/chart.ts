import type { Bucket, Metric } from "../types/sim";

export interface Frame {
  width: number;
  height: number;
  pad: number;
}

export interface Domain {
  maxX: number;
  maxY: number;
  log: boolean;
}

export interface Line {
  series: Bucket[];
  bucketMs: number;
}

export function niceMax(value: number): number {
  if (value <= 0) return 1;
  const magnitude = 10 ** Math.floor(Math.log10(value));
  const step = [1, 2, 5, 10].find((m) => m * magnitude >= value) ?? 10;
  return step * magnitude;
}

export function ratePerSecond(b: Bucket, metric: Metric, bucketMs: number): number {
  return (b[metric] * 1000) / bucketMs;
}

export function scaleY(value: number, domain: Domain, frame: Frame): number {
  const inner = frame.height - 2 * frame.pad;
  const ratio = domain.log
    ? Math.log10(1 + Math.max(0, value)) / Math.log10(1 + domain.maxY)
    : value / domain.maxY;
  return frame.height - frame.pad - Math.min(1, ratio) * inner;
}

export function scaleX(seconds: number, domain: Domain, frame: Frame): number {
  const inner = frame.width - 2 * frame.pad;
  return frame.pad + (Math.min(seconds, domain.maxX) / Math.max(1, domain.maxX)) * inner;
}

function visibleBuckets(line: Line, maxX: number): Bucket[] {
  return line.series.filter((b) => b.t_ms / 1000 <= maxX);
}

export function linePath(line: Line, metric: Metric, domain: Domain, frame: Frame): string {
  return visibleBuckets(line, domain.maxX)
    .map((b, i) => {
      const x = scaleX(b.t_ms / 1000, domain, frame).toFixed(1);
      const y = scaleY(ratePerSecond(b, metric, line.bucketMs), domain, frame).toFixed(1);
      return `${i === 0 ? "M" : "L"}${x},${y}`;
    })
    .join(" ");
}

export function domainFor(lines: Line[], metric: Metric, windowS: number | null, log: boolean): Domain {
  const longest = Math.max(1, ...lines.map((l) => Math.ceil((l.series.at(-1)?.t_ms ?? 0) / 1000)));
  const maxX = windowS === null ? longest : Math.min(windowS, longest);
  const visible = lines.flatMap((l) => visibleBuckets(l, maxX).map((b) => ratePerSecond(b, metric, l.bucketMs)));
  return { maxX, maxY: niceMax(Math.max(0, ...visible)), log };
}

export function yTicks(domain: Domain): number[] {
  if (!domain.log) return [0, 0.25, 0.5, 0.75, 1].map((f) => Math.round(f * domain.maxY));
  const ticks = [0];
  for (let v = 1; v <= domain.maxY; v *= 10) ticks.push(v);
  return ticks;
}
