import { api } from "../api/client";
import { useAsync } from "../hooks/useAsync";
import { percent } from "../lib/display";

export function Evaluation() {
  const { data, error } = useAsync(() => api.eval(), []);
  const max = Math.max(0.0001, ...(data ?? []).map((r) => r.recall_at_10));

  return (
    <div className="panel">
      <h2>Offline evaluation</h2>
      <p className="muted">
        For every seeded user the last 20% of watches are held out. Models are trained on the rest and each recommender
        returns 10 titles. Cold-start rows hide the whole history and only use the genres of the user's taste cluster as
        onboarding picks.
      </p>
      {error && <p className="error">{error}</p>}
      <table className="eval">
        <thead>
          <tr>
            <th>Scenario</th>
            <th>Algorithm</th>
            <th>Precision@10</th>
            <th>Recall@10</th>
            <th>Users</th>
          </tr>
        </thead>
        <tbody>
          {(data ?? []).map((r) => (
            <tr key={`${r.scenario}-${r.algorithm}`}>
              <td>{r.scenario}</td>
              <td>
                <strong>{r.algorithm}</strong>
              </td>
              <td>{percent(r.precision_at_10)}</td>
              <td>
                <div className="bar">
                  <div className={`fill ${r.algorithm}`} style={{ width: `${(r.recall_at_10 / max) * 100}%` }} />
                  <span>{percent(r.recall_at_10)}</span>
                </div>
              </td>
              <td>{r.users}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
