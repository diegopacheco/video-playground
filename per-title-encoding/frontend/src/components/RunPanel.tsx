import type { Analysis, RunStatus } from "../types/analysis";

interface Props {
  analysis: Analysis | null;
  status: RunStatus | null;
  onRun: () => void;
}

export function RunPanel({ analysis, status, onRun }: Props) {
  const running = status?.state === "running";
  const pct = status && status.total > 0 ? (status.done / status.total) * 100 : 0;
  return (
    <div className="run">
      <div className="muted">
        {analysis
          ? `${analysis.encoder} · metrics ${analysis.metrics.join(", ")} · analysed ${new Date(analysis.generated_at * 1000).toLocaleString()} in ${analysis.elapsed_secs.toFixed(0)}s`
          : "No cached analysis yet"}
      </div>
      <button onClick={onRun} disabled={running}>
        {running ? `Encoding ${status?.done}/${status?.total}` : "Re-run analysis"}
      </button>
      {running && (
        <div className="progress">
          <div style={{ width: `${pct}%` }} />
        </div>
      )}
      {status?.state === "failed" && <div className="error">Last run failed: {status.error}</div>}
    </div>
  );
}
