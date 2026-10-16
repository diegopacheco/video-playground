import type { Beacon, StitchView, UserProfile } from "../types";

async function json<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} failed with ${response.status}: ${await response.text()}`);
  }
  return (await response.json()) as T;
}

export function manifestUrl(user: string, session: string): string {
  return `/api/manifest.m3u8?user=${encodeURIComponent(user)}&session=${encodeURIComponent(session)}`;
}

export function fetchUsers(): Promise<UserProfile[]> {
  return json<UserProfile[]>("/api/users");
}

export function fetchStitch(user: string, session: string): Promise<StitchView> {
  return json<StitchView>(`/api/stitch?user=${encodeURIComponent(user)}&session=${encodeURIComponent(session)}`);
}

export function fetchBeacons(session: string): Promise<Beacon[]> {
  return json<Beacon[]>(`/api/beacons?session=${encodeURIComponent(session)}`);
}
