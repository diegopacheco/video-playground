import type { AuditEvent, Playback, Session } from "../types";

export class ApiError extends Error {
  constructor(public status: number, public code: string, message: string) {
    super(message);
  }
}

async function send<T>(path: string, init: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  const body = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new ApiError(res.status, body.error ?? "http", body.message ?? res.statusText);
  }
  return body as T;
}

function json(body: unknown, token?: string): RequestInit {
  const headers: Record<string, string> = { "content-type": "application/json" };
  if (token) headers.authorization = `Bearer ${token}`;
  return { method: "POST", headers, body: JSON.stringify(body) };
}

export function createSession(user: string, entitled: boolean, expired: boolean): Promise<Session> {
  return send("/api/session", json({ user, entitled, expired }));
}

export function createPlayback(token: string, content: string, expired: boolean): Promise<Playback> {
  return send("/api/playback", json({ content, expired }, token));
}

export function fetchAudit(): Promise<AuditEvent[]> {
  return send("/api/audit", { method: "GET" });
}

export const LICENSE_URL = "/api/license/clearkey";
