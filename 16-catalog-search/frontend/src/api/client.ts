import type { IndexStats, SearchResponse } from "../types/search";

async function getJson<T>(url: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(url, { signal });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null;
    throw new Error(body?.error ?? `request failed with ${response.status}`);
  }
  return (await response.json()) as T;
}

export function search(query: string, limit: number, signal?: AbortSignal): Promise<SearchResponse> {
  const params = new URLSearchParams({ q: query, limit: String(limit) });
  return getJson<SearchResponse>(`/api/search?${params.toString()}`, signal);
}

export function stats(): Promise<IndexStats> {
  return getJson<IndexStats>("/api/stats");
}
