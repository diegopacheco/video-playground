interface Props {
  values: number[];
  limitMs: number;
}

const W = 300;
const H = 80;

export function DriftChart({ values, limitMs }: Props) {
  const y = (v: number) => H / 2 - (Math.max(-limitMs, Math.min(limitMs, v)) / limitMs) * (H / 2 - 4);
  const step = values.length > 1 ? W / (values.length - 1) : W;
  const points = values.map((v, i) => `${(i * step).toFixed(1)},${y(v).toFixed(1)}`).join(" ");
  return (
    <svg className="chart" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" role="img" aria-label="drift history">
      <rect x="0" y={y(40)} width={W} height={y(-40) - y(40)} className="band" />
      <line x1="0" x2={W} y1={H / 2} y2={H / 2} className="axis" />
      <polyline points={points} className="line" />
    </svg>
  );
}
