export interface Resolution {
  width: number;
  height: number;
}

export interface Point {
  resolution: Resolution;
  target_kbps: number;
  bitrate_kbps: number;
  scores: Record<string, number>;
}

export interface LadderRung {
  target: number;
  reached: boolean;
  point: Point;
}

export interface GenericComparison {
  point: Point;
  equal_quality_kbps: number | null;
  saving_pct: number | null;
}

export interface Savings {
  generic_kbps: number;
  per_title_kbps: number;
  saving_pct: number;
  compared_rungs: number;
}

export interface TitleReport {
  id: string;
  name: string;
  description: string;
  points: Point[];
  hulls: Record<string, Point[]>;
  ladder: LadderRung[];
  generic: GenericComparison[];
  savings: Savings;
}

export interface Analysis {
  generated_at: number;
  elapsed_secs: number;
  encoder: string;
  metrics: string[];
  ladder_metric: string;
  targets: number[];
  titles: TitleReport[];
}

export type RunState = "idle" | "running" | "done" | "failed";

export interface RunStatus {
  state: RunState;
  done: number;
  total: number;
  error: string | null;
}
