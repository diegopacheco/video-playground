import { useState } from "react";
import type { Point, TitleReport } from "../types/analysis";
import { formatKbps, formatScore, linearScale, linearTicks, logScale, logTicks } from "../lib/scale";
import { resolutionLabel, resolutions } from "../lib/ladder";

export const SERIES_COLORS = ["#2a78d6", "#eb6834", "#1baf7a"];

const W = 640;
const H = 380;
const M = { top: 30, right: 18, bottom: 46, left: 66 };

interface Props {
  title: TitleReport;
  metric: string;
  targets: number[];
  showTargets: boolean;
}

interface Hover {
  point: Point;
  label: string;
  x: number;
  y: number;
}

export function RateQualityChart({ title, metric, targets, showTargets }: Props) {
  const [hover, setHover] = useState<Hover | null>(null);
  const kbps = title.points.map((p) => p.bitrate_kbps);
  const minK = Math.min(...kbps) * 0.85;
  const maxK = Math.max(...kbps) * 1.15;
  const scores = title.points.map((p) => p.scores[metric] ?? 0);
  const yMin = metric === "ssim" ? Math.floor(Math.min(...scores) * 20) / 20 : 0;
  const yMax = metric === "ssim" ? 1 : 100;
  const x = logScale([minK, maxK], [M.left, W - M.right]);
  const y = linearScale([yMin, yMax], [H - M.bottom, M.top]);
  const res = resolutions(title.points);
  const hull = title.hulls[metric] ?? [];
  const score = (p: Point) => p.scores[metric] ?? 0;
  const path = (ps: Point[]) => ps.map((p, i) => `${i === 0 ? "M" : "L"}${x(p.bitrate_kbps)},${y(score(p))}`).join(" ");
  const at = (p: Point, label: string) => setHover({ point: p, label, x: x(p.bitrate_kbps), y: y(score(p)) });

  return (
    <div className="chart">
      <div className="legend">
        {res.map((r, i) => (
          <span key={resolutionLabel(r)}>
            <i className="swatch dot" style={{ background: SERIES_COLORS[i % SERIES_COLORS.length] }} />
            {resolutionLabel(r)}
          </span>
        ))}
        <span>
          <i className="swatch line" />
          convex hull
        </span>
        <span>
          <i className="swatch ring" />
          per-title rung
        </span>
        <span>
          <i className="swatch square" />
          generic rung
        </span>
      </div>
      <div className="plot">
        <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`${title.name} ${metric} versus bitrate`}>
          {linearTicks(yMin, yMax, 5).map((t) => (
            <g key={`y${t}`}>
              <line x1={M.left} x2={W - M.right} y1={y(t)} y2={y(t)} className="grid" />
              <text x={M.left - 8} y={y(t) + 4} textAnchor="end" className="tick">
                {formatScore(metric, t)}
              </text>
            </g>
          ))}
          {logTicks(minK, maxK).map((t) => (
            <g key={`x${t}`}>
              <line x1={x(t)} x2={x(t)} y1={M.top} y2={H - M.bottom} className="grid" />
              <text x={x(t)} y={H - M.bottom + 16} textAnchor="middle" className="tick">
                {t >= 1000 ? `${t / 1000}M` : `${t}k`}
              </text>
            </g>
          ))}
          <text x={(M.left + W - M.right) / 2} y={H - 8} textAnchor="middle" className="axis-label">
            bitrate (log scale)
          </text>
          <text x={14} y={(M.top + H - M.bottom) / 2} textAnchor="middle" className="axis-label" transform={`rotate(-90 14 ${(M.top + H - M.bottom) / 2})`}>
            {metric.toUpperCase()}
          </text>
          {showTargets &&
            targets.map((t) => (
              <g key={`t${t}`}>
                <line x1={M.left} x2={W - M.right} y1={y(t)} y2={y(t)} className="target" />
                <text x={M.left + 4} y={y(t) - 4} className="target-label">
                  target {t}
                </text>
              </g>
            ))}
          {res.map((r, i) => {
            const ps = title.points
              .filter((p) => resolutionLabel(p.resolution) === resolutionLabel(r))
              .sort((a, b) => a.bitrate_kbps - b.bitrate_kbps);
            const color = SERIES_COLORS[i % SERIES_COLORS.length];
            return (
              <g key={resolutionLabel(r)}>
                <path d={path(ps)} fill="none" stroke={color} strokeWidth={2} />
                {ps.map((p) => (
                  <circle key={p.target_kbps} cx={x(p.bitrate_kbps)} cy={y(score(p))} r={4} fill={color} className="mark" />
                ))}
              </g>
            );
          })}
          <path d={path(hull)} fill="none" className="hull" />
          {title.generic.map((g) => (
            <rect
              key={`g${resolutionLabel(g.point.resolution)}${g.point.target_kbps}`}
              x={x(g.point.bitrate_kbps) - 6}
              y={y(score(g.point)) - 6}
              width={12}
              height={12}
              rx={2}
              className="generic"
            />
          ))}
          {title.ladder.map((r, i) => (
            <g key={`l${i}`}>
              <circle cx={x(r.point.bitrate_kbps)} cy={y(score(r.point))} r={9} className="rung" />
              <text x={x(r.point.bitrate_kbps)} y={y(score(r.point)) - 14} textAnchor="middle" className="rung-label">
                R{i + 1}
              </text>
            </g>
          ))}
          {title.points.map((p) => (
            <circle
              key={`h${resolutionLabel(p.resolution)}${p.target_kbps}`}
              cx={x(p.bitrate_kbps)}
              cy={y(score(p))}
              r={10}
              className="hit"
              onMouseEnter={() => at(p, resolutionLabel(p.resolution))}
              onMouseLeave={() => setHover(null)}
            />
          ))}
        </svg>
        {hover && (
          <div className="tooltip" style={{ left: `${(hover.x / W) * 100}%`, top: `${(hover.y / H) * 100}%` }}>
            <strong>
              {hover.label} @ {hover.point.target_kbps} kbps
            </strong>
            <div>measured {formatKbps(hover.point.bitrate_kbps)}</div>
            {Object.entries(hover.point.scores).map(([k, v]) => (
              <div key={k}>
                {k.toUpperCase()} {formatScore(k, v)}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
