export interface EdgeMetrics {
  requests: number;
  hits: number;
  misses: number;
  stale: number;
  coalesced: number;
  origin_fetches: number;
  origin_errors: number;
  revalidations: number;
  evictions: number;
  hit_ratio: number;
  origin_offload: number;
  cache_entries: number;
  cache_bytes: number;
  cache_capacity_bytes: number;
  in_flight: number;
}

export interface OriginConfig {
  latency_ms: number;
  segment_bytes: number;
  max_age: number;
  stale_while_revalidate: number;
}

export interface OriginStats {
  requests: number;
  unique_keys: number;
  config: OriginConfig;
}

export type CacheStatus = "HIT" | "MISS" | "STALE" | "ERROR";

export interface ProbeHit {
  status: CacheStatus;
  coalesced: boolean;
  millis: number;
}

export interface ProbeResult {
  key: string;
  hits: ProbeHit[];
  originRequestsDelta: number;
}
