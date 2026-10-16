import type { History, Mode, Sample } from "../types/latency";

export interface ChartGeometry {
  width: number;
  height: number;
  windowMs: number;
}

export interface ChartModel {
  yMax: number;
  ticks: number[];
  paths: Record<Mode, string>;
}

export function niceMax(value: number): number {
  const chunk = value <= 2000 ? 1000 : 2000;
  return Math.max(chunk, Math.ceil(value / chunk) * chunk);
}

function path(samples: Sample[], end: number, yMax: number, g: ChartGeometry): string {
  const start = end - g.windowMs;
  return samples
    .filter((s) => s.at >= start)
    .map((s, i) => {
      const x = ((s.at - start) / g.windowMs) * g.width;
      const y = g.height - (Math.min(s.latencyMs, yMax) / yMax) * g.height;
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

export function buildChart(history: History, end: number, g: ChartGeometry): ChartModel {
  const start = end - g.windowMs;
  const visible = [...history.ll, ...history.regular].filter((s) => s.at >= start);
  const yMax = niceMax(Math.max(0, ...visible.map((s) => s.latencyMs)));
  const step = yMax / 4;
  return {
    yMax,
    ticks: [0, 1, 2, 3, 4].map((i) => i * step),
    paths: {
      ll: path(history.ll, end, yMax, g),
      regular: path(history.regular, end, yMax, g),
    },
  };
}
