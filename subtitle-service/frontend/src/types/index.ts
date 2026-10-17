export type TagCounts = Record<string, number>;

export interface LossReport {
  cuesIn: number;
  cuesOut: number;
  kept: TagCounts;
  approximated: TagCounts;
  dropped: TagCounts;
  styleDropped: Record<string, string[]>;
  fidelity: number;
}

export interface TrackInfo {
  locale: string;
  name: string;
  assBytes: number;
  vttBytes: number;
  convertMicros: number;
  report: LossReport;
}

export type Mode = "native" | "jassub" | "burned";

export const MODES: Mode[] = ["native", "jassub", "burned"];
