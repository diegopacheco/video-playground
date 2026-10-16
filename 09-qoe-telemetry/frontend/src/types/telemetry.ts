export type Dimension = "cdn" | "title";

export interface WindowMetrics {
  sessions: number;
  events: number;
  startups: number;
  avg_startup_ms: number;
  p95_startup_ms: number;
  rebuffer_ratio: number;
  switches_per_min: number;
  error_rate: number;
  played_ms: number;
  rebuffer_ms: number;
}

export interface WindowResult {
  window_start: number;
  window_end: number;
  dimension: Dimension;
  key: string;
  metrics: WindowMetrics;
}

export interface WindowBatch {
  results: WindowResult[];
  consumed: number;
  late_dropped: number;
}

export type BeaconEvent =
  | { kind: "startup"; startup_ms: number }
  | { kind: "rebuffer_start" }
  | { kind: "rebuffer_end"; rebuffer_ms: number }
  | { kind: "bitrate_switch"; from_kbps: number; to_kbps: number }
  | { kind: "error"; code: string }
  | { kind: "heartbeat"; played_ms: number };

export type Beacon = {
  session_id: string;
  title: string;
  cdn: string;
  ts: number;
} & BeaconEvent;

export type MetricName = Exclude<keyof WindowMetrics, "played_ms" | "rebuffer_ms" | "events" | "startups">;
