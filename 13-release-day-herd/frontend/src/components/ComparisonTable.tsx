import { colorOf, compact, millis, percent } from "../lib/format";
import type { SimReport } from "../types/sim";

interface Props {
  reports: SimReport[];
}

export function ComparisonTable({ reports }: Props) {
  const bestSuccess = Math.max(...reports.map((r) => r.success_rate));
  const lowestOrigin = Math.min(...reports.map((r) => r.peak_origin_rps));
  return (
    <section className="card">
      <h2>Comparison</h2>
      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Strategy</th>
              <th>Success</th>
              <th>Peak origin rps</th>
              <th>Peak API rps</th>
              <th>p50 TTFF</th>
              <th>p95 TTFF</th>
              <th>p99 TTFF</th>
              <th>Retries</th>
              <th>Wasted retries</th>
              <th>Max admitted / s</th>
            </tr>
          </thead>
          <tbody>
            {reports.map((r) => (
              <tr key={r.strategy}>
                <td>
                  <i className="dot" style={{ background: colorOf(r.strategy) }} /> {r.strategy}
                </td>
                <td className={r.success_rate === bestSuccess ? "good" : r.success_rate < 0.5 ? "bad" : ""}>{percent(r.success_rate)}</td>
                <td className={r.peak_origin_rps === lowestOrigin ? "good" : ""}>{compact(r.peak_origin_rps)}</td>
                <td>{compact(r.peak_api_rps)}</td>
                <td>{millis(r.ttff_p50_ms)}</td>
                <td>{millis(r.ttff_p95_ms)}</td>
                <td>{millis(r.ttff_p99_ms)}</td>
                <td>{compact(r.retries)}</td>
                <td className={r.wasted_retries === 0 ? "good" : ""}>{compact(r.wasted_retries)}</td>
                <td>{r.max_admitted_per_s ? compact(r.max_admitted_per_s) : "-"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
