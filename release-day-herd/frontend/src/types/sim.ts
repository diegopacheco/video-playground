export interface SimConfig {
  users: number;
  seed: number;
  press_window_ms: number;
  api_rps: number;
  origin_rps: number;
  admission_rps: number;
  edges: number;
  renditions: number;
  api_latency_ms: number;
  api_reject_ms: number;
  edge_latency_ms: number;
  origin_latency_ms: number;
  origin_reject_ms: number;
  jitter_ms: number;
  max_attempts: number;
  give_up_ms: number;
  backoff_base_ms: number;
  backoff_cap_ms: number;
}

export interface Bucket {
  t_ms: number;
  api_requests: number;
  api_rejected: number;
  origin_requests: number;
  origin_rejected: number;
  first_frames: number;
  admitted: number;
  retries: number;
}

export type Metric = "origin_requests" | "api_requests" | "first_frames" | "retries" | "admitted";

export interface SimReport {
  strategy: string;
  bucket_ms: number;
  users: number;
  succeeded: number;
  abandoned: number;
  success_rate: number;
  peak_origin_rps: number;
  peak_api_rps: number;
  max_admitted_per_s: number;
  origin_requests: number;
  api_requests: number;
  retries: number;
  wasted_retries: number;
  prewarmed_objects: number;
  ttff_p50_ms: number | null;
  ttff_p95_ms: number | null;
  ttff_p99_ms: number | null;
  series: Bucket[];
}

export interface JoinResponse {
  ticket: number;
  position: number;
  eta_ms: number;
}

export interface StatusResponse {
  ticket: number;
  position: number;
  eta_ms: number;
  admitted: boolean;
  token: string | null;
}

export interface PlayResponse {
  ticket: number;
  manifest: string;
}

export interface RoomStats {
  rate_per_s: number;
  joined: number;
  admitted: number;
  waiting: number;
  played: number;
  max_admitted_in_any_second: number;
  max_plays_in_any_second: number;
}
