export interface TraceSummary {
  id: string;
  name: string;
  description: string;
  duration_s: number;
  mean_kbps: number;
  min_kbps: number;
  max_kbps: number;
  bandwidth_kbps: number[];
}

export interface AlgorithmInfo {
  id: string;
  name: string;
  description: string;
}

export interface PlayerConfig {
  ladder_kbps: number[];
  segment_s: number;
  segments: number;
  max_buffer_s: number;
  startup_buffer_s: number;
  rtt_s: number;
}

export interface SegmentRecord {
  index: number;
  quality: number;
  bitrate_kbps: number;
  request_s: number;
  buffer_start_s: number;
  done_s: number;
  throughput_kbps: number;
  buffer_s: number;
  stall_s: number;
  idle_s: number;
}

export interface Session {
  segments: SegmentRecord[];
  startup_delay_s: number;
  rebuffer_s: number;
  rebuffer_events: number;
  content_s: number;
  end_s: number;
}

export interface Metrics {
  avg_bitrate_kbps: number;
  switch_count: number;
  rebuffer_s: number;
  rebuffer_events: number;
  rebuffer_ratio: number;
  startup_delay_s: number;
  qoe: number;
}

export interface AlgorithmRun {
  algorithm: string;
  name: string;
  metrics: Metrics;
  session: Session;
}

export interface SimulationReport {
  trace: TraceSummary;
  config: PlayerConfig;
  runs: AlgorithmRun[];
}

export interface Point {
  x: number;
  y: number;
}

export interface Series {
  id: string;
  label: string;
  color: string;
  points: Point[];
  dashed?: boolean;
}
