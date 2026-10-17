import type { TitleReport } from "../types/analysis";
import { formatKbps, formatPct, formatScore } from "../lib/scale";
import { resolutionLabel } from "../lib/ladder";

interface Props {
  title: TitleReport;
  metric: string;
}

export function LadderTable({ title, metric }: Props) {
  return (
    <div className="ladders">
      <table>
        <caption>Per-title ladder (cheapest hull point meeting each {metric.toUpperCase()} target)</caption>
        <thead>
          <tr>
            <th>Rung</th>
            <th>Target</th>
            <th>Resolution</th>
            <th>Bitrate</th>
            <th>{metric.toUpperCase()}</th>
          </tr>
        </thead>
        <tbody>
          {title.ladder.map((r, i) => (
            <tr key={i}>
              <td>R{i + 1}</td>
              <td>
                {r.target}
                {!r.reached && <span className="flag"> not reached</span>}
              </td>
              <td>{resolutionLabel(r.point.resolution)}</td>
              <td className="num">{formatKbps(r.point.bitrate_kbps)}</td>
              <td className="num">{formatScore(metric, r.point.scores[metric] ?? 0)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <table>
        <caption>Generic ladder and the hull bitrate for the same quality</caption>
        <thead>
          <tr>
            <th>Resolution</th>
            <th>Bitrate</th>
            <th>{metric.toUpperCase()}</th>
            <th>Hull at same quality</th>
            <th>Saving</th>
          </tr>
        </thead>
        <tbody>
          {title.generic.map((g, i) => (
            <tr key={i}>
              <td>{resolutionLabel(g.point.resolution)}</td>
              <td className="num">{formatKbps(g.point.bitrate_kbps)}</td>
              <td className="num">{formatScore(metric, g.point.scores[metric] ?? 0)}</td>
              <td className="num">{g.equal_quality_kbps === null ? "n/a" : formatKbps(g.equal_quality_kbps)}</td>
              <td className="num">{g.saving_pct === null ? "n/a" : formatPct(g.saving_pct)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
