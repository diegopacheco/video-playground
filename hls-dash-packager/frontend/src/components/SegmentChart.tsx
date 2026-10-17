import type { RenditionReport } from "../types/manifest";
import { formatBytes, formatKbps, maxSegmentBytes } from "../lib/format";

export function SegmentChart({ renditions }: { renditions: RenditionReport[] }) {
  const max = maxSegmentBytes(renditions);
  return (
    <div className="chart">
      {renditions.map((r) => (
        <div className="chart-row" key={r.id}>
          <span className="chart-label strong">{r.id}</span>
          <div className="bars">
            {r.segments.map((s) => (
              <div
                key={s.index}
                className={`bar rung-${r.height}`}
                style={{ height: `${Math.max(2, (s.bytes / max) * 100)}%` }}
                title={`${s.uri}: ${formatBytes(s.bytes)}, ${formatKbps(s.kbps)}, ${s.duration}s`}
              />
            ))}
          </div>
          <span className="chart-value small">{formatBytes(Math.max(...r.segments.map((s) => s.bytes)))} max</span>
        </div>
      ))}
    </div>
  );
}
