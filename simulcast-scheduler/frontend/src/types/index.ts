export type StatusLabel = "available" | "locked" | "geo_blocked";

export type BlockReason = "not_licensed" | "license_expired" | "unknown_region";

export interface LocalTime {
  timezone: string;
  local: string;
  abbrev: string;
  utc_offset: string;
}

export interface Entitlement {
  episode_id: string;
  country: string;
  status: StatusLabel;
  unlock_at: string | null;
  available_until: string | null;
  countdown_ms: number | null;
  reason: BlockReason | null;
  local_release: LocalTime | null;
  server_now: string;
}

export interface EpisodeBoard {
  id: string;
  title: string;
  release_utc: string;
  regions: Entitlement[];
}

export interface Board {
  server_now: string;
  offset_ms: number;
  episodes: EpisodeBoard[];
}

export interface Metrics {
  cache_hits: number;
  cache_misses: number;
  throttled: number;
  plays_granted: number;
  plays_throttled: number;
  warmed: number;
  cache_entries: number;
  bucket_tokens: number;
}

export interface ProbeResult {
  status: number;
  retryAfterMs: number | null;
  label: string;
}
