export interface Claims {
  sub: string;
  ent: string[];
  exp: number;
}

export interface Session {
  token: string;
  claims: Claims;
}

export interface Playback {
  manifestUrl: string;
  expires: number;
}

export interface AuditEvent {
  at: number;
  kind: string;
  outcome: "granted" | "denied";
  detail: string;
}

export interface Scenario {
  id: string;
  title: string;
  description: string;
  entitled: boolean;
  sessionExpired: boolean;
  urlExpired: boolean;
  tamper: boolean;
  expect: string;
}

export type LogLevel = "info" | "ok" | "error";

export interface LogEntry {
  id: number;
  level: LogLevel;
  text: string;
}

export interface LicenseExchange {
  request: string;
  response: string;
}
