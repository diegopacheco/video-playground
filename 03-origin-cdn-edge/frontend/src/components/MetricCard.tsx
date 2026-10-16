interface Props {
  label: string;
  value: string;
  hint?: string;
  tone?: "hit" | "miss" | "stale" | "neutral";
}

export function MetricCard({ label, value, hint, tone = "neutral" }: Props) {
  return (
    <div className={`card metric ${tone}`}>
      <div className="metric-label">{label}</div>
      <div className="metric-value">{value}</div>
      {hint ? <div className="metric-hint">{hint}</div> : null}
    </div>
  );
}
