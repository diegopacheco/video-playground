import type { Board, Metrics, ProbeResult } from "../types";

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return (await res.json()) as T;
}

function post<T>(url: string, body?: unknown): Promise<T> {
  return fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  }).then((r) => json<T>(r));
}

export const api = {
  board: () => fetch("/api/board").then((r) => json<Board>(r)),
  metrics: () => fetch("/api/metrics").then((r) => json<Metrics>(r)),
  resetMetrics: () => post<Metrics>("/api/metrics/reset"),
  setOffset: (offsetMs: number) => post<unknown>("/api/clock", { offset_ms: offsetMs }),
  release: (id: string, inSecs: number) => post<unknown>(`/api/episodes/${id}/release`, { in_secs: inSecs }),
  play: async (id: string, country: string): Promise<ProbeResult> => {
    const res = await fetch(`/api/episodes/${id}/play`, { method: "POST", headers: { "x-country": country } });
    const body = (await res.json()) as { status?: string; session_id?: string };
    const retry = res.headers.get("x-retry-after-ms");
    return {
      status: res.status,
      retryAfterMs: retry === null ? null : Number(retry),
      label: body.session_id ? "session granted" : (body.status ?? "unknown"),
    };
  },
};
