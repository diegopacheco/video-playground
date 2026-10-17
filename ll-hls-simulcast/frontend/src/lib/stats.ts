import type { Sample, Summary } from "../types/latency";

export function percentile(sorted: number[], p: number): number | null {
  if (sorted.length === 0) return null;
  const rank = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.min(sorted.length - 1, Math.max(0, rank))];
}

export function summarize(samples: Sample[]): Summary {
  const values = samples.map((s) => s.latencyMs);
  const sorted = [...values].sort((a, b) => a - b);
  const total = values.reduce((sum, v) => sum + v, 0);
  return {
    count: values.length,
    current: values.length ? values[values.length - 1] : null,
    min: sorted.length ? sorted[0] : null,
    avg: values.length ? total / values.length : null,
    p50: percentile(sorted, 50),
    p95: percentile(sorted, 95),
  };
}

export function appendBounded(samples: Sample[], sample: Sample, limit: number): Sample[] {
  const next = [...samples, sample];
  return next.length > limit ? next.slice(next.length - limit) : next;
}

export function latencyMs(now: number, playingDate: Date | null): number | null {
  if (!playingDate) return null;
  const value = now - playingDate.getTime();
  return Number.isFinite(value) && value >= 0 ? value : null;
}

export function formatMs(value: number | null): string {
  return value === null ? "-" : `${Math.round(value)} ms`;
}
