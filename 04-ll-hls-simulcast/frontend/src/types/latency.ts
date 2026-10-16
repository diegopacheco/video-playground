export type Mode = "ll" | "regular";

export interface Sample {
  at: number;
  latencyMs: number;
}

export interface Summary {
  count: number;
  current: number | null;
  min: number | null;
  avg: number | null;
  p50: number | null;
  p95: number | null;
}

export type History = Record<Mode, Sample[]>;
