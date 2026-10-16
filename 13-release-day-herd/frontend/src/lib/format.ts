export function compact(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(n >= 10_000_000 ? 0 : 1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(n >= 10_000 ? 0 : 1)}k`;
  return String(n);
}

export function percent(ratio: number): string {
  return `${(ratio * 100).toFixed(2)}%`;
}

export function millis(ms: number | null): string {
  if (ms === null) return "-";
  return ms >= 10_000 ? `${(ms / 1000).toFixed(1)} s` : `${ms} ms`;
}

export const strategyColors: Record<string, string> = {
  naive: "#d9485f",
  "jittered-backoff": "#e8890c",
  "cdn-prewarm": "#7a5af8",
  "waiting-room": "#1c7ed6",
  combined: "#2f9e44",
};

export function colorOf(strategy: string): string {
  return strategyColors[strategy] ?? "#495057";
}
