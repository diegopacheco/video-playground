import type { AlgorithmRun, Metrics } from "../types/abr";

export type Better = "higher" | "lower";

export interface Column {
  key: keyof Metrics;
  label: string;
  better: Better;
  format: (value: number) => string;
}

export const COLUMNS: Column[] = [
  { key: "qoe", label: "QoE score", better: "higher", format: (v) => v.toFixed(3) },
  { key: "avg_bitrate_kbps", label: "Avg bitrate (kbps)", better: "higher", format: (v) => v.toFixed(0) },
  { key: "rebuffer_ratio", label: "Rebuffer ratio", better: "lower", format: (v) => `${(v * 100).toFixed(2)}%` },
  { key: "rebuffer_s", label: "Rebuffer (s)", better: "lower", format: (v) => v.toFixed(2) },
  { key: "rebuffer_events", label: "Stalls", better: "lower", format: (v) => v.toFixed(0) },
  { key: "switch_count", label: "Switches", better: "lower", format: (v) => v.toFixed(0) },
  { key: "startup_delay_s", label: "Startup (s)", better: "lower", format: (v) => v.toFixed(2) },
];

export function bestRuns(runs: AlgorithmRun[], column: Column): Set<string> {
  if (runs.length < 2) return new Set();
  const values = runs.map((r) => r.metrics[column.key]);
  const target = column.better === "higher" ? Math.max(...values) : Math.min(...values);
  const winners = runs.filter((r) => Math.abs(r.metrics[column.key] - target) < 1e-9);
  return winners.length === runs.length ? new Set() : new Set(winners.map((r) => r.algorithm));
}
