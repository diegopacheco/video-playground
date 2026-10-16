import type { Analysis, RunStatus } from "../types/analysis";

async function json<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as { error?: string };
    throw new Error(body.error ?? `HTTP ${response.status}`);
  }
  return (await response.json()) as T;
}

export async function fetchAnalysis(): Promise<Analysis | null> {
  const response = await fetch("/api/analysis");
  if (response.status === 404) return null;
  return json<Analysis>(response);
}

export async function fetchStatus(): Promise<RunStatus> {
  return json<RunStatus>(await fetch("/api/status"));
}

export async function startAnalysis(): Promise<RunStatus> {
  return json<RunStatus>(await fetch("/api/analysis", { method: "POST" }));
}
