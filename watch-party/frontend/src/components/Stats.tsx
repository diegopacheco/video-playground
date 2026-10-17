import type { SyncStats } from "../hooks/useSync";
import type { ClockSample } from "../sync/clock";
import { DEADBAND_MS } from "../sync/drift";
import { DriftChart } from "./DriftChart";

interface Props {
  clock: ClockSample | null;
  samples: number;
  sync: SyncStats;
  skewMs: number;
}

const ms = (v: number | undefined) => (v === undefined ? "-" : `${v.toFixed(1)} ms`);

export function Stats({ clock, samples, sync, skewMs }: Props) {
  const inSync = Math.abs(sync.drift) <= DEADBAND_MS;
  return (
    <section className="card">
      <h3>Sync</h3>
      <dl className="stats">
        <dt>Clock offset</dt>
        <dd data-testid="offset">{ms(clock?.offset)}</dd>
        <dt>RTT (best of {samples})</dt>
        <dd data-testid="rtt">{ms(clock?.rtt)}</dd>
        <dt>Drift</dt>
        <dd data-testid="drift" className={inSync ? "good" : "bad"}>
          {ms(sync.drift)}
        </dd>
        <dt>Correction</dt>
        <dd data-testid="correction">{sync.correction}</dd>
        <dt>Playback rate</dt>
        <dd>{sync.playbackRate.toFixed(3)}x</dd>
        <dt>Expected / actual</dt>
        <dd>
          {sync.expected.toFixed(3)}s / {sync.actual.toFixed(3)}s
        </dd>
        {skewMs !== 0 && (
          <>
            <dt>Simulated clock skew</dt>
            <dd>{skewMs} ms</dd>
          </>
        )}
      </dl>
      <DriftChart values={sync.history} limitMs={200} />
      <p className="muted small">Drift over the last 15s, band is the +/-{DEADBAND_MS} ms deadband.</p>
    </section>
  );
}
