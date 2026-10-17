import { useMemo, useState } from "react";
import { domainFor, linePath, scaleX, scaleY, yTicks } from "../lib/chart";
import { colorOf, compact } from "../lib/format";
import type { Metric, SimReport } from "../types/sim";

interface Props {
  reports: SimReport[];
  originCapacity: number;
  apiCapacity: number;
}

const metrics: { key: Metric; label: string }[] = [
  { key: "origin_requests", label: "Origin requests / s" },
  { key: "api_requests", label: "Playback API requests / s" },
  { key: "first_frames", label: "First frames / s" },
  { key: "retries", label: "Retries / s" },
  { key: "admitted", label: "Waiting room admissions / s" },
];

const windows: { value: number | null; label: string }[] = [
  { value: 10, label: "first 10 s" },
  { value: 60, label: "first 60 s" },
  { value: null, label: "all" },
];

const frame = { width: 900, height: 340, pad: 48 };

function toLine(r: SimReport) {
  return { series: r.series, bucketMs: r.bucket_ms };
}

export function LoadChart({ reports, originCapacity, apiCapacity }: Props) {
  const [metric, setMetric] = useState<Metric>("origin_requests");
  const [windowS, setWindowS] = useState<number | null>(10);
  const [log, setLog] = useState(true);
  const [hidden, setHidden] = useState<Set<string>>(new Set());

  const visible = reports.filter((r) => !hidden.has(r.strategy));
  const domain = useMemo(
    () => domainFor(visible.map(toLine), metric, windowS, log),
    [visible, metric, windowS, log],
  );
  const capacity = metric === "origin_requests" ? originCapacity : metric === "api_requests" ? apiCapacity : null;
  const xTicks = Array.from({ length: 6 }, (_, i) => Math.round((domain.maxX * i) / 5));

  const toggle = (name: string) =>
    setHidden((h) => {
      const next = new Set(h);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });

  return (
    <section className="card">
      <div className="chart-head">
        <h2>Load over time</h2>
        <div className="controls">
          <select value={metric} onChange={(e) => setMetric(e.target.value as Metric)}>
            {metrics.map((m) => (
              <option key={m.key} value={m.key}>
                {m.label}
              </option>
            ))}
          </select>
          <select value={String(windowS)} onChange={(e) => setWindowS(e.target.value === "null" ? null : Number(e.target.value))}>
            {windows.map((w) => (
              <option key={w.label} value={String(w.value)}>
                {w.label}
              </option>
            ))}
          </select>
          <label className="check">
            <input type="checkbox" checked={log} onChange={(e) => setLog(e.target.checked)} /> log scale
          </label>
        </div>
      </div>
      <svg viewBox={`0 0 ${frame.width} ${frame.height}`} className="chart" role="img" aria-label="load over time">
        {yTicks(domain).map((v) => (
          <g key={`y${v}`}>
            <line x1={frame.pad} x2={frame.width - frame.pad} y1={scaleY(v, domain, frame)} y2={scaleY(v, domain, frame)} className="grid" />
            <text x={frame.pad - 6} y={scaleY(v, domain, frame) + 4} textAnchor="end" className="tick">
              {compact(v)}
            </text>
          </g>
        ))}
        {xTicks.map((s) => (
          <text key={`x${s}`} x={scaleX(s, domain, frame)} y={frame.height - frame.pad + 18} textAnchor="middle" className="tick">
            {s} s
          </text>
        ))}
        {capacity !== null && capacity <= domain.maxY && (
          <g>
            <line
              x1={frame.pad}
              x2={frame.width - frame.pad}
              y1={scaleY(capacity, domain, frame)}
              y2={scaleY(capacity, domain, frame)}
              className="capacity"
            />
            <text x={frame.width - frame.pad} y={scaleY(capacity, domain, frame) - 6} textAnchor="end" className="capacity-label">
              capacity {compact(capacity)}/s
            </text>
          </g>
        )}
        {visible.map((r) => (
          <path key={r.strategy} d={linePath(toLine(r), metric, domain, frame)} stroke={colorOf(r.strategy)} className="series" />
        ))}
      </svg>
      <div className="legend">
        {reports.map((r) => (
          <button
            key={r.strategy}
            className={hidden.has(r.strategy) ? "pill off" : "pill"}
            style={{ borderColor: colorOf(r.strategy) }}
            onClick={() => toggle(r.strategy)}
          >
            <i style={{ background: colorOf(r.strategy) }} /> {r.strategy}
          </button>
        ))}
      </div>
    </section>
  );
}
