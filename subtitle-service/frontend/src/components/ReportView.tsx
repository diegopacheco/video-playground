import { useEffect, useState } from "react";
import { fetchMaster } from "../api/client";
import { formatBytes, formatCounts, formatPercent } from "../lib/format";
import type { TrackInfo } from "../types";

export function ReportView({ tracks }: { tracks: TrackInfo[] }) {
  const [master, setMaster] = useState("loading");
  useEffect(() => {
    fetchMaster().then(setMaster, (e: unknown) => setMaster(String(e)));
  }, []);

  return (
    <div className="report">
      <section className="card">
        <h2>Conversion lossiness per locale</h2>
        <table>
          <thead>
            <tr>
              <th>Locale</th>
              <th>ASS</th>
              <th>VTT</th>
              <th>Cues in/out</th>
              <th>Fidelity</th>
              <th>Kept</th>
              <th>Approximated</th>
              <th>Dropped</th>
            </tr>
          </thead>
          <tbody>
            {tracks.map((t) => (
              <tr key={t.locale}>
                <td>
                  <b>{t.locale}</b> {t.name}
                </td>
                <td>{formatBytes(t.assBytes)}</td>
                <td>{formatBytes(t.vttBytes)}</td>
                <td>
                  {t.report.cuesIn}/{t.report.cuesOut}
                </td>
                <td>
                  <span className="meter">
                    <span style={{ width: formatPercent(t.report.fidelity) }} />
                  </span>
                  {formatPercent(t.report.fidelity)}
                </td>
                <td className="tags kept">{formatCounts(t.report.kept)}</td>
                <td className="tags approx">{formatCounts(t.report.approximated)}</td>
                <td className="tags dropped">{formatCounts(t.report.dropped)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="note">
          Style properties WebVTT cannot carry:{" "}
          {Object.entries(tracks[0]?.report.styleDropped ?? {})
            .map(([style, props]) => `${style} (${props.join(", ")})`)
            .join("; ")}
        </p>
      </section>
      <section className="card">
        <h2>HLS master with EXT-X-MEDIA TYPE=SUBTITLES</h2>
        <pre>{master}</pre>
      </section>
    </div>
  );
}
