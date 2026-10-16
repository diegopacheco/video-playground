import type { TitleReport } from "../types/analysis";
import { formatKbps, formatScore } from "../lib/scale";
import { resolutionLabel } from "../lib/ladder";

interface Props {
  title: TitleReport;
  metrics: string[];
  ladderMetric: string;
}

export function PointsTable({ title, metrics, ladderMetric }: Props) {
  const hull = new Set((title.hulls[ladderMetric] ?? []).map((p) => `${resolutionLabel(p.resolution)}@${p.target_kbps}`));
  const rows = [...title.points].sort((a, b) => a.resolution.height - b.resolution.height || a.target_kbps - b.target_kbps);
  return (
    <table className="points">
      <caption>{title.name}: every encode in the grid</caption>
      <thead>
        <tr>
          <th>Resolution</th>
          <th>Budget</th>
          <th>Measured</th>
          {metrics.map((m) => (
            <th key={m}>{m.toUpperCase()}</th>
          ))}
          <th>On {ladderMetric.toUpperCase()} hull</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((p) => {
          const key = `${resolutionLabel(p.resolution)}@${p.target_kbps}`;
          return (
            <tr key={key} className={hull.has(key) ? "on-hull" : undefined}>
              <td>{resolutionLabel(p.resolution)}</td>
              <td className="num">{p.target_kbps} kbps</td>
              <td className="num">{formatKbps(p.bitrate_kbps)}</td>
              {metrics.map((m) => (
                <td key={m} className="num">
                  {formatScore(m, p.scores[m] ?? 0)}
                </td>
              ))}
              <td>{hull.has(key) ? "yes" : ""}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
