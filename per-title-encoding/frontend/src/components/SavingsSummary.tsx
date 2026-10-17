import type { TitleReport } from "../types/analysis";
import { formatKbps, formatPct, formatScore } from "../lib/scale";
import { ladderTotals, percentChange } from "../lib/ladder";

interface Props {
  titles: TitleReport[];
  metric: string;
}

export function SavingsSummary({ titles, metric }: Props) {
  const totalsOf = titles.map((t) => ladderTotals(t, metric));
  const maxKbps = Math.max(...totalsOf.flatMap((s) => [s.genericKbps, s.perTitleKbps]), 1);
  return (
    <div className="tiles">
      {titles.map((t, i) => {
        const totals = totalsOf[i];
        const change = percentChange(totals.genericKbps, totals.perTitleKbps);
        return (
          <section key={t.id} className="tile">
            <h3>{t.name}</h3>
            <div className="hero">{formatPct(t.savings.saving_pct)}</div>
            <div className="muted">
              fewer bits for the same {metric.toUpperCase()} as the generic ladder ({t.savings.compared_rungs} rungs:{" "}
              {formatKbps(t.savings.generic_kbps)} vs {formatKbps(t.savings.per_title_kbps)})
            </div>
            <div className="bars">
              <div className="bar-row">
                <span>generic</span>
                <div className="bar generic-bar" style={{ width: `${(totals.genericKbps / maxKbps) * 100}%` }} />
                <span className="num">{formatKbps(totals.genericKbps)}</span>
              </div>
              <div className="bar-row">
                <span>per-title</span>
                <div className="bar title-bar" style={{ width: `${(totals.perTitleKbps / maxKbps) * 100}%` }} />
                <span className="num">{formatKbps(totals.perTitleKbps)}</span>
              </div>
            </div>
            <div className="muted">
              Whole ladder: {change <= 0 ? `${(-change).toFixed(1)}% smaller` : `${change.toFixed(1)}% larger`}, top quality{" "}
              {formatScore(metric, totals.perTitleTop)} vs {formatScore(metric, totals.genericTop)}
            </div>
          </section>
        );
      })}
    </div>
  );
}
