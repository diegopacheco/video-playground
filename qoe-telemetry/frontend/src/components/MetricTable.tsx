import { cell, latestWindow, series } from "../lib/metrics";
import type { Dimension, MetricName, WindowResult } from "../types/telemetry";
import { Sparkline } from "./Sparkline";

const COLUMNS: { metric: MetricName; label: string }[] = [
  { metric: "sessions", label: "Sessions" },
  { metric: "avg_startup_ms", label: "Avg startup" },
  { metric: "p95_startup_ms", label: "P95 startup" },
  { metric: "rebuffer_ratio", label: "Rebuffer ratio" },
  { metric: "switches_per_min", label: "Switches/min" },
  { metric: "error_rate", label: "Error rate" },
];

interface Props {
  title: string;
  dimension: Dimension;
  results: WindowResult[];
  trend: MetricName;
}

export function MetricTable({ title, dimension, results, trend }: Props) {
  const rows = latestWindow(results, dimension);
  const window = rows[0];
  return (
    <section className="card">
      <header className="card-head">
        <h2>{title}</h2>
        <span className="muted">
          {window
            ? `window ${new Date(window.window_start).toLocaleTimeString()} - ${new Date(window.window_end).toLocaleTimeString()}`
            : "waiting for the first closed window"}
        </span>
      </header>
      <table>
        <thead>
          <tr>
            <th>{dimension === "cdn" ? "CDN" : "Title"}</th>
            {COLUMNS.map((c) => (
              <th key={c.metric}>{c.label}</th>
            ))}
            <th>Trend ({COLUMNS.find((c) => c.metric === trend)?.label})</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.key}>
              <td className="key">{r.key}</td>
              {COLUMNS.map((c) => {
                const { text, tone } = cell(c.metric, r.metrics);
                return (
                  <td key={c.metric} className={tone ?? ""}>
                    {text}
                  </td>
                );
              })}
              <td>
                <Sparkline values={series(results, dimension, r.key, trend)} color="#5b6ee1" />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
