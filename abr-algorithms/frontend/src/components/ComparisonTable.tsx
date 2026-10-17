import { COLUMNS, bestRuns } from "../lib/compare";
import { algorithmColor } from "../lib/colors";
import type { AlgorithmRun } from "../types/abr";

export function ComparisonTable({ runs }: { runs: AlgorithmRun[] }) {
  const winners = COLUMNS.map((c) => bestRuns(runs, c));
  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Algorithm</th>
            {COLUMNS.map((c) => (
              <th key={c.key}>
                {c.label}
                <span className="better">{c.better === "higher" ? "higher is better" : "lower is better"}</span>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {runs.map((r) => (
            <tr key={r.algorithm}>
              <td>
                <span className="dot" style={{ background: algorithmColor(r.algorithm) }} />
                {r.name}
              </td>
              {COLUMNS.map((c, i) => (
                <td key={c.key} className={winners[i]?.has(r.algorithm) ? "best" : undefined}>
                  {c.format(r.metrics[c.key])}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
