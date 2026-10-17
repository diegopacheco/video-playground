import type { Beacon, WindowBatch, WindowResult } from "../types/telemetry";

export async function postBeacons(beacons: Beacon[]): Promise<void> {
  const res = await fetch("/api/beacons", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(beacons),
  });
  if (!res.ok) {
    throw new Error(`ingest rejected beacons: ${res.status} ${await res.text()}`);
  }
}

export async function fetchHistory(): Promise<WindowResult[]> {
  const res = await fetch("/api/windows");
  if (!res.ok) {
    throw new Error(`history unavailable: ${res.status}`);
  }
  return (await res.json()) as WindowResult[];
}

export function openWindowStream(
  onBatch: (batch: WindowBatch) => void,
  onStatus: (connected: boolean) => void,
): () => void {
  const source = new EventSource("/api/stream");
  source.onopen = () => onStatus(true);
  source.onerror = () => onStatus(false);
  source.addEventListener("windows", (e) => {
    onBatch(JSON.parse((e as MessageEvent<string>).data) as WindowBatch);
  });
  return () => source.close();
}
