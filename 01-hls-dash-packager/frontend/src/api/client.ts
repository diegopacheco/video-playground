import type { Manifest } from "../types/manifest";

export const API_URL: string = import.meta.env.VITE_API_URL ?? "http://localhost:8101";

export function mediaUrl(path: string): string {
  return `${API_URL}/media/${path}`;
}

export async function fetchManifest(): Promise<Manifest> {
  const res = await fetch(`${API_URL}/api/manifest`);
  if (!res.ok) {
    throw new Error(`manifest request failed with ${res.status}`);
  }
  return (await res.json()) as Manifest;
}
