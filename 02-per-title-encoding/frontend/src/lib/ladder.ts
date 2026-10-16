import type { Point, Resolution, TitleReport } from "../types/analysis";

export function resolutionLabel(r: Resolution): string {
  return `${r.width}x${r.height}`;
}

export function resolutions(points: Point[]): Resolution[] {
  const seen = new Map<string, Resolution>();
  for (const p of points) seen.set(resolutionLabel(p.resolution), p.resolution);
  return [...seen.values()].sort((a, b) => a.height - b.height);
}

export interface LadderTotals {
  perTitleKbps: number;
  genericKbps: number;
  perTitleTop: number;
  genericTop: number;
}

export function ladderTotals(title: TitleReport, metric: string): LadderTotals {
  const perTitle = title.ladder.map((r) => r.point);
  const generic = title.generic.map((g) => g.point);
  const sum = (ps: Point[]) => ps.reduce((acc, p) => acc + p.bitrate_kbps, 0);
  const top = (ps: Point[]) => ps.reduce((acc, p) => Math.max(acc, p.scores[metric] ?? 0), 0);
  return {
    perTitleKbps: sum(perTitle),
    genericKbps: sum(generic),
    perTitleTop: top(perTitle),
    genericTop: top(generic),
  };
}

export function percentChange(from: number, to: number): number {
  return from === 0 ? 0 : ((to - from) / from) * 100;
}
