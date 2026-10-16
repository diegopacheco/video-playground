import type { Breakdown } from "../types/search";

export interface ScorePart {
  key: "similarity" | "coverage" | "popularity";
  label: string;
  value: number;
  share: number;
}

export function scoreParts(b: Breakdown): ScorePart[] {
  const raw: Omit<ScorePart, "share">[] = [
    { key: "similarity", label: "similarity", value: b.similarity_score },
    { key: "coverage", label: "coverage", value: b.coverage_score },
    { key: "popularity", label: "popularity", value: b.popularity_score },
  ];
  const sum = raw.reduce((acc, p) => acc + p.value, 0);
  return raw.map((p) => ({ ...p, share: sum > 0 ? (p.value / sum) * 100 : 0 }));
}

export function percent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function latency(us: number): string {
  return us < 1000 ? `${us} µs` : `${(us / 1000).toFixed(2)} ms`;
}
