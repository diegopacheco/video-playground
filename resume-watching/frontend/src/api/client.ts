import type { OfferResult, Playhead, RowItem, Stats } from "../types";

async function call<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  if (!res.ok) {
    throw new Error(`${init?.method ?? "GET"} ${path} failed with ${res.status}`);
  }
  return res.status === 204 ? (undefined as T) : ((await res.json()) as T);
}

export async function sendHeartbeat(p: Playhead): Promise<OfferResult> {
  const body = await call<{ result: OfferResult }>("/api/heartbeat", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(p),
  });
  return body.result;
}

export function fetchRow(profile: string): Promise<RowItem[]> {
  return call<RowItem[]>(`/api/profiles/${encodeURIComponent(profile)}/continue-watching`);
}

export function fetchStats(): Promise<Stats> {
  return call<Stats>("/api/stats");
}

export function flushNow(): Promise<{ written: number }> {
  return call<{ written: number }>("/api/flush", { method: "POST" });
}

export function resetProfile(profile: string): Promise<void> {
  return call<void>(`/api/profiles/${encodeURIComponent(profile)}`, { method: "DELETE" });
}
