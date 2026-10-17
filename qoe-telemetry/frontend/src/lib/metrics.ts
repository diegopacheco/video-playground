import type { Dimension, MetricName, WindowMetrics, WindowResult } from "../types/telemetry";

export type Health = "good" | "warn" | "bad";

const THRESHOLDS: Record<MetricName, [number, number] | null> = {
  sessions: null,
  avg_startup_ms: [2000, 4000],
  p95_startup_ms: [3000, 6000],
  rebuffer_ratio: [0.01, 0.05],
  switches_per_min: [4, 8],
  error_rate: [0.02, 0.1],
};

export function health(metric: MetricName, value: number): Health | null {
  const t = THRESHOLDS[metric];
  if (!t) return null;
  if (value >= t[1]) return "bad";
  if (value >= t[0]) return "warn";
  return "good";
}

export function mergeWindows(current: WindowResult[], incoming: WindowResult[], keep: number): WindowResult[] {
  const seen = new Map<string, WindowResult>();
  for (const r of [...current, ...incoming]) {
    seen.set(`${r.window_start}|${r.dimension}|${r.key}`, r);
  }
  const starts = [...new Set([...seen.values()].map((r) => r.window_start))].sort((a, b) => a - b);
  const cutoff = starts[Math.max(0, starts.length - keep)] ?? 0;
  return [...seen.values()]
    .filter((r) => r.window_start >= cutoff)
    .sort((a, b) => a.window_start - b.window_start || a.key.localeCompare(b.key));
}

export function latestWindow(results: WindowResult[], dimension: Dimension): WindowResult[] {
  const rows = results.filter((r) => r.dimension === dimension);
  const last = Math.max(...rows.map((r) => r.window_start), -Infinity);
  return rows.filter((r) => r.window_start === last).sort((a, b) => a.key.localeCompare(b.key));
}

export function keysOf(results: WindowResult[], dimension: Dimension): string[] {
  return [...new Set(results.filter((r) => r.dimension === dimension).map((r) => r.key))].sort();
}

export function series(results: WindowResult[], dimension: Dimension, key: string, metric: MetricName): number[] {
  return results
    .filter((r) => r.dimension === dimension && r.key === key)
    .map((r) => r.metrics[metric]);
}

export function cell(metric: MetricName, m: WindowMetrics): { text: string; tone: Health | null } {
  const startupMetric = metric === "avg_startup_ms" || metric === "p95_startup_ms";
  if (startupMetric && m.startups === 0) return { text: "-", tone: null };
  return { text: format(metric, m[metric]), tone: health(metric, m[metric]) };
}

export function format(metric: MetricName, value: number): string {
  if (metric === "rebuffer_ratio" || metric === "error_rate") return `${(value * 100).toFixed(1)}%`;
  if (metric === "avg_startup_ms" || metric === "p95_startup_ms") return `${Math.round(value)} ms`;
  if (metric === "switches_per_min") return value.toFixed(1);
  return String(value);
}
