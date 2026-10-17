import type { CacheStatus, ProbeHit } from "../types/metrics";

export function pushBounded<T>(history: readonly T[], value: T, max: number): T[] {
  const next = [...history, value];
  return next.length > max ? next.slice(next.length - max) : next;
}

export interface Sample {
  requests: number;
  hits: number;
  stale: number;
}

export function windowHitRatio(previous: Sample | null, current: Sample): number | null {
  if (previous === null || current.requests < previous.requests) return null;
  const requests = current.requests - previous.requests;
  if (requests === 0) return null;
  const served = current.hits - previous.hits + (current.stale - previous.stale);
  return served / requests;
}

export function countStatuses(hits: readonly ProbeHit[]): Record<CacheStatus, number> {
  const counts: Record<CacheStatus, number> = { HIT: 0, MISS: 0, STALE: 0, ERROR: 0 };
  for (const hit of hits) counts[hit.status] += 1;
  return counts;
}
