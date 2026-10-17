import type { Stats } from "../types";

interface Props {
  stats: Stats | null;
  onFlush: () => void;
  onReset: () => void;
}

export function StatsBar({ stats, onFlush, onReset }: Props) {
  const cells: [string, string][] = stats
    ? [
        ["store", stats.store],
        ["flush every", `${stats.flush_secs}s`],
        ["heartbeats", String(stats.heartbeats)],
        ["store writes", String(stats.writes)],
        ["reduction", stats.writes ? `${stats.reduction.toFixed(1)}x` : "-"],
        ["stale dropped", String(stats.stale)],
        ["pending", String(stats.pending)],
      ]
    : [];
  return (
    <section className="stats" data-testid="stats">
      {cells.map(([k, v]) => (
        <div key={k} className="stat">
          <span>{k}</span>
          <strong>{v}</strong>
        </div>
      ))}
      <div className="actions">
        <button onClick={onFlush}>Flush now</button>
        <button className="ghost" onClick={onReset}>
          Reset profile
        </button>
      </div>
    </section>
  );
}
