import { cellText } from "../lib/time";
import type { Board } from "../types";

export function StatusBoard({ board }: { board: Board }) {
  const countries = board.episodes[0]?.regions.map((r) => r.country) ?? [];
  return (
    <section className="panel">
      <table className="board">
        <thead>
          <tr>
            <th>Episode</th>
            {countries.map((c) => (
              <th key={c}>{c}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {board.episodes.map((e) => (
            <tr key={e.id}>
              <td>
                <strong>{e.title}</strong>
                <div className="muted small">{e.release_utc.replace("T", " ").slice(0, 16)} UTC</div>
              </td>
              {e.regions.map((r) => (
                <td key={r.country} className={`cell ${r.status} ${r.reason ?? ""}`} title={r.local_release ? `${r.local_release.local} ${r.local_release.abbrev}` : ""}>
                  {cellText(r)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="legend small">
        <span className="cell available">OPEN</span>
        <span className="cell locked">countdown</span>
        <span className="cell geo_blocked">BLOCKED</span>
        <span className="cell geo_blocked license_expired">EXPIRED</span>
      </div>
    </section>
  );
}
