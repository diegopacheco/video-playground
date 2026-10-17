import { formatPercent } from "../lib/format";

interface Props {
  ratio: number;
  label: string;
}

const RADIUS = 52;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function HitRatioGauge({ ratio, label }: Props) {
  const clamped = Math.min(Math.max(Number.isFinite(ratio) ? ratio : 0, 0), 1);
  return (
    <div className="card gauge">
      <svg viewBox="0 0 140 140" width="160" height="160" role="img" aria-label={`${label} ${formatPercent(clamped)}`}>
        <circle cx="70" cy="70" r={RADIUS} className="gauge-track" />
        <circle
          cx="70"
          cy="70"
          r={RADIUS}
          className="gauge-fill"
          strokeDasharray={`${clamped * CIRCUMFERENCE} ${CIRCUMFERENCE}`}
          transform="rotate(-90 70 70)"
        />
        <text x="70" y="76" textAnchor="middle" className="gauge-text">
          {formatPercent(clamped)}
        </text>
      </svg>
      <div className="metric-label">{label}</div>
    </div>
  );
}
