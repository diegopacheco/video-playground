import { useCallback, useState } from "react";
import { api } from "../api/client";
import { usePoll } from "../hooks/usePoll";
import type { Board, ProbeResult } from "../types";

const COUNTRIES = ["US", "BR", "GB", "DE", "IN", "JP", "KR", "CN", "AU"];

interface Summary {
  total: number;
  byOutcome: Record<string, number>;
  minRetry: number | null;
  maxRetry: number | null;
}

function summarize(results: ProbeResult[]): Summary {
  const byOutcome: Record<string, number> = {};
  const retries = results
    .filter((r) => r.status === 429)
    .map((r) => r.retryAfterMs)
    .filter((n): n is number => n !== null);
  for (const r of results) {
    const key = `${r.status} ${r.label}`;
    byOutcome[key] = (byOutcome[key] ?? 0) + 1;
  }
  return {
    total: results.length,
    byOutcome,
    minRetry: retries.length ? Math.min(...retries) : null,
    maxRetry: retries.length ? Math.max(...retries) : null,
  };
}

export function HerdPanel({ board }: { board: Board }) {
  const loadMetrics = useCallback(() => api.metrics(), []);
  const { data: metrics, refresh } = usePoll(loadMetrics, 1000);
  const [summary, setSummary] = useState<Summary | null>(null);
  const episodeId = (board.episodes.find((e) => e.id === "neon-harbor-s1e1") ?? board.episodes[0])?.id ?? "";

  const burst = async () => {
    const calls = Array.from({ length: 300 }, (_, i) => api.play(episodeId, COUNTRIES[i % COUNTRIES.length]));
    setSummary(summarize(await Promise.all(calls)));
    refresh();
  };

  const tiles: Array<[string, number | string]> = metrics
    ? [
        ["cache hits", metrics.cache_hits],
        ["cache misses", metrics.cache_misses],
        ["throttled (429)", metrics.throttled],
        ["plays granted", metrics.plays_granted],
        ["plays throttled", metrics.plays_throttled],
        ["warmed entries", metrics.warmed],
        ["cache entries", metrics.cache_entries],
        ["bucket tokens", metrics.bucket_tokens.toFixed(1)],
      ]
    : [];

  return (
    <section className="panel">
      <div className="tiles">
        {tiles.map(([label, value]) => (
          <div key={label} className="tile">
            <span className="muted small">{label}</span>
            <strong>{value}</strong>
          </div>
        ))}
      </div>
      <div className="row">
        <button onClick={burst}>Fire 300 parallel play requests</button>
        <button className="ghost" onClick={() => api.resetMetrics().then(refresh)}>
          Reset counters
        </button>
      </div>
      {summary && (
        <div className="burst">
          <h3>Burst of {summary.total}</h3>
          <ul>
            {Object.entries(summary.byOutcome).map(([k, v]) => (
              <li key={k}>
                <code>{k}</code> {v}
              </li>
            ))}
          </ul>
          {summary.minRetry !== null && (
            <p className="muted small">
              429 Retry-After jittered between {summary.minRetry} ms and {summary.maxRetry} ms
            </p>
          )}
        </div>
      )}
    </section>
  );
}
