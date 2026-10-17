interface Props {
  values: number[];
  color: string;
}

export function Sparkline({ values, color }: Props) {
  const width = 120;
  const height = 28;
  if (values.length < 2) return <svg width={width} height={height} />;
  const max = Math.max(...values, 1e-9);
  const step = width / (values.length - 1);
  const points = values
    .map((v, i) => `${(i * step).toFixed(1)},${(height - 2 - (v / max) * (height - 4)).toFixed(1)}`)
    .join(" ");
  return (
    <svg width={width} height={height} className="spark">
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.6" />
    </svg>
  );
}
