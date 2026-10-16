import type { JoinResponse, PlayResponse, RoomStats, SimConfig, SimReport, StatusResponse } from "../types/sim";

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    ...init,
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { error?: string };
    throw new Error(body.error ?? `${res.status} ${res.statusText}`);
  }
  return (await res.json()) as T;
}

export const api = {
  defaultConfig: () => request<SimConfig>("/api/config/default"),
  compare: (config: SimConfig) =>
    request<SimReport[]>("/api/compare", { method: "POST", body: JSON.stringify({ config }) }),
  join: () => request<JoinResponse>("/room/join", { method: "POST" }),
  status: (ticket: number) => request<StatusResponse>(`/room/status/${ticket}`),
  play: (token: string) => request<PlayResponse>(`/room/play?token=${encodeURIComponent(token)}`),
  stats: () => request<RoomStats>("/room/stats"),
  reset: () => request<RoomStats>("/room/reset", { method: "POST" }),
};
