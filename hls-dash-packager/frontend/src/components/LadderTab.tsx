import type { Manifest } from "../types/manifest";
import { bitsPerPixel, formatBytes, formatKbps, peakToAverage } from "../lib/format";
import { SegmentChart } from "./SegmentChart";
import { KeyframeTimeline } from "./KeyframeTimeline";

export function LadderTab({ manifest }: { manifest: Manifest }) {
  const { alignment } = manifest;
  return (
    <div className="stack">
      <div className="stats">
        <Stat label="Frame rate" value={`${manifest.fps} fps`} />
        <Stat label="GOP" value={`${manifest.gop_frames} frames / ${manifest.gop_seconds}s`} />
        <Stat label="Segment" value={`${manifest.segment_seconds}s CMAF`} />
        <Stat label="Duration" value={`${manifest.duration.toFixed(1)}s`} />
        <Stat label="Keyframes aligned" value={alignment.aligned ? "yes" : "no"} ok={alignment.aligned} />
        <Stat label="Segments start on IDR" value={alignment.boundaries_on_keyframes ? "yes" : "no"} ok={alignment.boundaries_on_keyframes} />
      </div>

      <section className="card">
        <h2>ABR ladder</h2>
        <table>
          <thead>
            <tr>
              <th>Rendition</th>
              <th>Resolution</th>
              <th>Codecs</th>
              <th>Target</th>
              <th>Average</th>
              <th>Peak segment</th>
              <th>Peak/avg</th>
              <th>Bits/pixel</th>
              <th>Segments</th>
              <th>Init</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {manifest.renditions.map((r) => (
              <tr key={r.id}>
                <td className="strong">{r.id}</td>
                <td>{r.width}x{r.height}</td>
                <td className="mono">{r.codecs}</td>
                <td>{formatKbps(r.target_kbps)}</td>
                <td>{formatKbps(r.average_kbps)}</td>
                <td>{formatKbps(r.peak_kbps)}</td>
                <td>{peakToAverage(r).toFixed(2)}</td>
                <td>{bitsPerPixel(r, manifest.fps).toFixed(3)}</td>
                <td>{r.segments.length}</td>
                <td>{formatBytes(r.init_bytes)}</td>
                <td>{formatBytes(r.total_bytes)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="card">
        <h2>Segment sizes</h2>
        <p className="muted">Each bar is one {manifest.segment_seconds}s fMP4 segment. Same duration on every rung, so size is bitrate.</p>
        <SegmentChart renditions={manifest.renditions} />
      </section>

      <section className="card">
        <h2>Keyframe alignment</h2>
        <p className="muted">
          Dots are IDR frames probed from each encode, lines are segment boundaries. A switch between rungs only works where every rung has an IDR at the same instant.
        </p>
        <KeyframeTimeline manifest={manifest} />
        {alignment.issues.length > 0 && (
          <ul className="issues">
            {alignment.issues.map((i) => (
              <li key={i}>{i}</li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function Stat({ label, value, ok }: { label: string; value: string; ok?: boolean }) {
  const tone = ok === undefined ? "" : ok ? "good" : "bad";
  return (
    <div className={`card stat ${tone}`}>
      <span className="muted small">{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
