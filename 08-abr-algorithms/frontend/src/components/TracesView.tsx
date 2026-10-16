import { bandwidthSteps } from "../lib/chart";
import { BANDWIDTH_COLOR } from "../lib/colors";
import type { TraceSummary } from "../types/abr";
import { TimeChart } from "./TimeChart";

export function TracesView({ traces }: { traces: TraceSummary[] }) {
  return (
    <div className="trace-grid">
      {traces.map((t) => (
        <section key={t.id} className="panel">
          <h2>{t.name}</h2>
          <p className="hint">{t.description}</p>
          <div className="stats">
            <span>mean {Math.round(t.mean_kbps)} kbps</span>
            <span>min {Math.round(t.min_kbps)} kbps</span>
            <span>max {Math.round(t.max_kbps)} kbps</span>
            <span>{t.duration_s}s, loops</span>
          </div>
          <TimeChart
            series={[{ id: t.id, label: t.name, color: BANDWIDTH_COLOR, points: bandwidthSteps(t.bandwidth_kbps, t.duration_s) }]}
            xMax={t.duration_s}
            yMax={t.max_kbps * 1.1}
            yLabel="kbps"
            height={200}
          />
        </section>
      ))}
    </div>
  );
}
