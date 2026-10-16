export type Plan = "basic" | "fan" | "mega";

export type Policy = "reject_newest" | "kick_oldest";

export interface Account {
  id: string;
  plan: Plan;
  limit: number;
}

export interface Lease {
  device: string;
  started_at_ms: number;
  expires_at_ms: number;
}

export interface Snapshot {
  account: Account;
  policy: Policy;
  ttl_ms: number;
  now_ms: number;
  leases: Lease[];
}

export type AcquireResult =
  | { status: "granted"; lease: Lease; evicted: string[] }
  | { status: "rejected"; active: number; limit: number };

export type DeviceStatus = "idle" | "playing" | "rejected" | "kicked" | "crashed";

export interface DeviceState {
  status: DeviceStatus;
  beats: number;
}

export interface LogEntry {
  at: number;
  text: string;
  tone: "ok" | "warn" | "bad" | "info";
}
