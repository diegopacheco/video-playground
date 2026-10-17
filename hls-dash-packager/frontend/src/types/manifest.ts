export interface SegmentInfo {
  index: number;
  uri: string;
  start: number;
  duration: number;
  bytes: number;
  kbps: number;
}

export interface RenditionReport {
  id: string;
  width: number;
  height: number;
  target_kbps: number;
  codecs: string;
  playlist: string;
  init_bytes: number;
  total_bytes: number;
  average_kbps: number;
  peak_kbps: number;
  keyframes: number[];
  segments: SegmentInfo[];
}

export interface AlignmentReport {
  aligned: boolean;
  boundaries_on_keyframes: boolean;
  keyframe_times: number[];
  issues: string[];
}

export interface Manifest {
  fps: number;
  gop_frames: number;
  gop_seconds: number;
  segment_seconds: number;
  duration: number;
  hls_master: string;
  dash_mpd: string;
  renditions: RenditionReport[];
  alignment: AlignmentReport;
}

export interface Level {
  index: number;
  width: number;
  height: number;
  bitrate: number;
}

export interface PlayerState {
  levels: Level[];
  current: number | null;
  selected: number;
  switches: string[];
  setLevel: (index: number) => void;
}
