import { useId } from "react";
import { linearScale, niceTicks, toPath } from "../lib/chart";
import type { Series } from "../types/abr";

export interface Band {
  from: number;
  to: number;
  color: string;
}

interface Props {
  series: Series[];
  xMax: number;
  yMax: number;
  yLabel: string;
  bands?: Band[];
  guides?: number[];
  height?: number;
  compact?: boolean;
}

const WIDTH = 960;

export function TimeChart({ series, xMax, yMax, yLabel, bands = [], guides = [], height = 280, compact = false }: Props) {
  const clipId = useId();
  const margin = compact ? { top: 6, right: 6, bottom: 6, left: 6 } : { top: 16, right: 20, bottom: 36, left: 64 };
  const innerRight = WIDTH - margin.right;
  const innerBottom = height - margin.bottom;
  const sx = linearScale([0, xMax], [margin.left, innerRight]);
  const sy = linearScale([0, yMax], [innerBottom, margin.top]);
  const yTicks = niceTicks(yMax, 4);
  const xTicks = niceTicks(xMax, 8);

  return (
    <svg className="chart" viewBox={`0 0 ${WIDTH} ${height}`} role="img" aria-label={yLabel}>
      <defs>
        <clipPath id={clipId}>
          <rect x={margin.left} y={margin.top} width={innerRight - margin.left} height={innerBottom - margin.top} />
        </clipPath>
      </defs>
      {!compact &&
        yTicks.map((t) => (
          <g key={`y${t}`}>
            <line x1={margin.left} x2={innerRight} y1={sy(t)} y2={sy(t)} className="grid" />
            <text x={margin.left - 8} y={sy(t) + 4} textAnchor="end" className="tick">
              {t}
            </text>
          </g>
        ))}
      {!compact &&
        xTicks.map((t) => (
          <text key={`x${t}`} x={sx(t)} y={innerBottom + 18} textAnchor="middle" className="tick">
            {t}s
          </text>
        ))}
      <g clipPath={`url(#${clipId})`}>
        {bands.map((b, i) => (
          <rect
            key={`b${i}`}
            x={sx(b.from)}
            y={margin.top}
            width={Math.max(3, sx(b.to) - sx(b.from))}
            height={innerBottom - margin.top}
            fill={b.color}
            opacity={0.35}
          />
        ))}
        {guides.map((g) => (
          <line key={`g${g}`} x1={margin.left} x2={innerRight} y1={sy(g)} y2={sy(g)} className="guide" />
        ))}
        {series.map((s) => (
          <path
            key={s.id}
            d={toPath(s.points, sx, sy)}
            fill="none"
            stroke={s.color}
            strokeWidth={compact ? 1.5 : 2}
            strokeDasharray={s.dashed ? "4 3" : undefined}
            strokeLinejoin="round"
          />
        ))}
      </g>
      <line x1={margin.left} x2={innerRight} y1={innerBottom} y2={innerBottom} className="axis" />
      {!compact && (
        <text x={14} y={margin.top + (innerBottom - margin.top) / 2} className="axis-label" transform={`rotate(-90 14 ${margin.top + (innerBottom - margin.top) / 2})`} textAnchor="middle">
          {yLabel}
        </text>
      )}
    </svg>
  );
}
