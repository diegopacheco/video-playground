import { useState } from "react";
import type { MetricName } from "../types/telemetry";
import type { WindowsState } from "../hooks/useWindows";
import { MetricTable } from "./MetricTable";

const TRENDS: { metric: MetricName; label: string }[] = [
  { metric: "rebuffer_ratio", label: "Rebuffer ratio" },
  { metric: "avg_startup_ms", label: "Avg startup" },
  { metric: "p95_startup_ms", label: "P95 startup" },
  { metric: "switches_per_min", label: "Switches/min" },
  { metric: "error_rate", label: "Error rate" },
];

export function Dashboard({ state }: { state: WindowsState }) {
  const [trend, setTrend] = useState<MetricName>("rebuffer_ratio");
  const windows = new Set(state.results.map((r) => r.window_start)).size;
  return (
    <div className="stack">
      <div className="stats">
        <Stat label="Stream" value={state.connected ? "live (SSE)" : "disconnected"} tone={state.connected ? "good" : "bad"} />
        <Stat label="Beacons consumed from Kafka" value={state.consumed.toLocaleString()} />
        <Stat label="Late beacons dropped" value={state.lateDropped.toLocaleString()} />
        <Stat label="Closed 10s windows kept" value={String(windows)} />
        <label className="stat">
          <span className="muted">Trend metric</span>
          <select value={trend} onChange={(e) => setTrend(e.target.value as MetricName)}>
            {TRENDS.map((t) => (
              <option key={t.metric} value={t.metric}>
                {t.label}
              </option>
            ))}
          </select>
        </label>
      </div>
      {state.error && <p className="bad-text">{state.error}</p>}
      <MetricTable title="Per CDN" dimension="cdn" results={state.results} trend={trend} />
      <MetricTable title="Per title" dimension="title" results={state.results} trend={trend} />
    </div>
  );
}

function Stat({ label, value, tone }: { label: string; value: string; tone?: string }) {
  return (
    <div className="stat">
      <span className="muted">{label}</span>
      <strong className={tone ?? ""}>{value}</strong>
    </div>
  );
}
