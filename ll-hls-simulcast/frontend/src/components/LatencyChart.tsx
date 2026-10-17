import { buildChart } from "../lib/chart";
import type { History } from "../types/latency";

const WIDTH = 720;
const HEIGHT = 240;
const PAD = 52;
const WINDOW_MS = 60_000;

interface Props {
  history: History;
}

export function LatencyChart({ history }: Props) {
  const latest = Math.max(0, ...[...history.ll, ...history.regular].map((s) => s.at));
  const model = buildChart(history, latest, { width: WIDTH, height: HEIGHT, windowMs: WINDOW_MS });
  return (
    <svg className="chart" viewBox={`0 0 ${WIDTH + PAD + 12} ${HEIGHT + 36}`} role="img" aria-label="Glass-to-glass latency over the last 60 seconds">
      <g transform={`translate(${PAD},10)`}>
        {model.ticks.map((t) => {
          const y = HEIGHT - (t / model.yMax) * HEIGHT;
          return (
            <g key={t}>
              <line x1={0} x2={WIDTH} y1={y} y2={y} className="grid" />
              <text x={-8} y={y + 4} textAnchor="end" className="tick">
                {t / 1000}s
              </text>
            </g>
          );
        })}
        <path d={model.paths.regular} className="line regular" />
        <path d={model.paths.ll} className="line ll" />
        <text x={0} y={HEIGHT + 22} className="tick">-60s</text>
        <text x={WIDTH} y={HEIGHT + 22} textAnchor="end" className="tick">now</text>
      </g>
    </svg>
  );
}
