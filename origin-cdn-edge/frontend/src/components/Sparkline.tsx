interface Props {
  title: string;
  values: number[];
  max?: number;
  format: (value: number) => string;
}

const WIDTH = 600;
const HEIGHT = 120;

export function Sparkline({ title, values, max, format }: Props) {
  const top = max ?? Math.max(1, ...values);
  const step = values.length > 1 ? WIDTH / (values.length - 1) : WIDTH;
  const points = values.map((v, i) => `${(i * step).toFixed(1)},${(HEIGHT - (v / top) * HEIGHT).toFixed(1)}`).join(" ");
  const last = values.length > 0 ? values[values.length - 1] : undefined;
  return (
    <div className="card spark">
      <div className="spark-head">
        <span className="metric-label">{title}</span>
        <span className="spark-last">{last === undefined ? "waiting" : format(last)}</span>
      </div>
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} preserveAspectRatio="none" className="spark-svg">
        <line x1="0" y1={HEIGHT / 2} x2={WIDTH} y2={HEIGHT / 2} className="spark-grid" />
        {values.length > 1 ? <polyline points={points} className="spark-line" /> : null}
      </svg>
    </div>
  );
}
