export interface Playhead {
  user_id: string;
  profile_id: string;
  device_id: string;
  title_id: string;
  episode_id: string;
  position_secs: number;
  duration_secs: number;
  client_ts_ms: number;
}

export type Source = "store" | "buffer";

export interface RowItem extends Playhead {
  progress: number;
  source: Source;
}

export type OfferResult = "buffered" | "replaced" | "stale";

export interface Stats {
  store: string;
  flush_secs: number;
  heartbeats: number;
  stale: number;
  writes: number;
  flushes: number;
  pending: number;
  reduction: number;
}

export interface Title {
  id: string;
  name: string;
  episode: string;
  duration: number;
  hue: number;
}

export interface DeviceState {
  titleId: string;
  position: number;
  playing: boolean;
  skewMs: number;
  sent: number;
  lastResult: OfferResult | "error" | null;
}
