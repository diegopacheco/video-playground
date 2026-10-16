import { latency } from "../lib/score";
import type { SearchResponse } from "../types/search";

export function QueryInfo({ response }: { response: SearchResponse }) {
  return (
    <div className="query-info">
      <span>normalized <code>{response.normalized}</code></span>
      <span>{response.grams.length} n-grams</span>
      <span>{response.candidates} candidates</span>
      <span>{response.results.length} ranked</span>
      <span>server {latency(response.took_us)}</span>
      <div className="grams">
        {response.grams.map((g) => (
          <code key={g}>{g}</code>
        ))}
      </div>
    </div>
  );
}
