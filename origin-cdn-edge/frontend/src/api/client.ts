import type { EdgeMetrics, OriginStats, ProbeHit } from "../types/metrics";

async function json<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`${url} returned ${response.status}`);
  }
  return (await response.json()) as T;
}

export function fetchEdgeMetrics(): Promise<EdgeMetrics> {
  return json<EdgeMetrics>("/edge/metrics");
}

export function fetchOriginStats(): Promise<OriginStats> {
  return json<OriginStats>("/origin/stats");
}

export async function resetCounters(): Promise<void> {
  await Promise.all([
    json<EdgeMetrics>("/edge/metrics/reset", { method: "POST" }),
    json<OriginStats>("/origin/stats/reset", { method: "POST" }),
  ]);
}

export async function fetchOriginKeyRequests(key: string): Promise<number> {
  const stats = await json<{ requests: number }>(`/origin/stats/key/${encodeURIComponent(key)}`);
  return stats.requests;
}

export async function probeBurst(key: string, n: number): Promise<ProbeHit[]> {
  const report = await json<{ results: ProbeHit[] }>(`/edge/probe/${encodeURIComponent(key)}?n=${n}`, { method: "POST" });
  return report.results;
}
