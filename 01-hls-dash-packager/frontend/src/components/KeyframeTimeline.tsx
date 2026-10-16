import type { Manifest } from "../types/manifest";

export function KeyframeTimeline({ manifest }: { manifest: Manifest }) {
  const width = 900;
  const rowHeight = 34;
  const left = 60;
  const usable = width - left - 10;
  const x = (t: number) => left + (t / manifest.duration) * usable;
  const height = manifest.renditions.length * rowHeight + 24;
  const ticks = Array.from({ length: Math.floor(manifest.duration / manifest.segment_seconds) + 1 }, (_, i) => i * manifest.segment_seconds);

  return (
    <svg className="timeline" viewBox={`0 0 ${width} ${height}`} role="img" aria-label="keyframe timeline">
      {manifest.renditions.map((r, row) => {
        const y = row * rowHeight + 16;
        return (
          <g key={r.id}>
            <text x={0} y={y + 4} className="tl-label">
              {r.id}
            </text>
            <line x1={left} x2={left + usable} y1={y} y2={y} className="tl-axis" />
            {r.segments.map((s) => (
              <line key={s.index} x1={x(s.start)} x2={x(s.start)} y1={y - 10} y2={y + 10} className="tl-boundary" />
            ))}
            {r.keyframes.map((k) => (
              <circle key={k} cx={x(k)} cy={y} r={4} className="tl-key" />
            ))}
          </g>
        );
      })}
      {ticks.map((t) => (
        <text key={t} x={x(t)} y={height - 2} className="tl-tick" textAnchor="middle">
          {t}s
        </text>
      ))}
    </svg>
  );
}
