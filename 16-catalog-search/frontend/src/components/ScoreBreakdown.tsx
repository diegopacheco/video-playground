import { percent, scoreParts } from "../lib/score";
import type { Breakdown } from "../types/search";

export function ScoreBreakdown({ breakdown }: { breakdown: Breakdown }) {
  const parts = scoreParts(breakdown);
  return (
    <div className="breakdown">
      <div className="bar" style={{ width: `${Math.min(breakdown.total, 1) * 100}%` }}>
        {parts.map((p) => (
          <span key={p.key} className={`bar-${p.key}`} style={{ width: `${p.share}%` }} title={`${p.label} ${p.value.toFixed(3)}`} />
        ))}
      </div>
      <dl>
        <div><dt>matched on</dt><dd>{breakdown.field}</dd></div>
        <div><dt>similarity</dt><dd>{percent(breakdown.similarity)}</dd></div>
        <div><dt>coverage</dt><dd>{percent(breakdown.coverage)}</dd></div>
        <div><dt>edit distance</dt><dd>{breakdown.edit_distance}</dd></div>
        <div><dt>n-gram overlap</dt><dd>{percent(breakdown.overlap)}</dd></div>
        <div><dt>field boost</dt><dd>×{breakdown.boost.toFixed(2)}</dd></div>
        <div><dt>popularity</dt><dd>{percent(breakdown.popularity)}</dd></div>
      </dl>
      <p className="formula">
        <span className="dot bar-similarity" /> {breakdown.similarity_score.toFixed(3)}
        {" + "}
        <span className="dot bar-coverage" /> {breakdown.coverage_score.toFixed(3)}
        {" + "}
        <span className="dot bar-popularity" /> {breakdown.popularity_score.toFixed(3)}
        {" = "}
        <strong>{breakdown.total.toFixed(3)}</strong>
      </p>
    </div>
  );
}
