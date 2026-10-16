import type { AlgorithmInfo, SimulationReport, TraceSummary } from "../types/abr";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, init);
  const body = await response.json();
  if (!response.ok) {
    throw new Error(body.error ?? `request failed with ${response.status}`);
  }
  return body as T;
}

export function fetchTraces(): Promise<TraceSummary[]> {
  return request("/api/traces");
}

export function fetchAlgorithms(): Promise<AlgorithmInfo[]> {
  return request("/api/algorithms");
}

export function simulate(trace: string, algorithms: string[]): Promise<SimulationReport> {
  return request("/api/simulate", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ trace, algorithms }),
  });
}
