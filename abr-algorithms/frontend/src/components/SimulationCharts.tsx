import { bandwidthSteps, bitrateSteps, bufferPoints, stallIntervals } from "../lib/chart";
import { BANDWIDTH_COLOR, STALL_COLOR, algorithmColor } from "../lib/colors";
import type { Series, SimulationReport } from "../types/abr";
import { Legend } from "./Legend";
import { TimeChart } from "./TimeChart";

export function SimulationCharts({ report }: { report: SimulationReport }) {
  const { config, runs, trace } = report;
  const xMax = Math.max(...runs.map((r) => r.session.end_s));
  const top = config.ladder_kbps[config.ladder_kbps.length - 1] ?? 1;
  const bitrateMax = top * 1.6;

  const bitrateSeries: Series[] = [
    { id: "bandwidth", label: "Trace bandwidth", color: BANDWIDTH_COLOR, points: bandwidthSteps(trace.bandwidth_kbps, xMax), dashed: true },
    ...runs.map((r) => ({ id: r.algorithm, label: r.name, color: algorithmColor(r.algorithm), points: bitrateSteps(r.session.segments) })),
  ];
  const bufferSeries: Series[] = runs.map((r) => ({
    id: r.algorithm,
    label: r.name,
    color: algorithmColor(r.algorithm),
    points: bufferPoints(r.session.segments, config.segment_s),
  }));
  const stalls = runs.flatMap((r) => stallIntervals(r.session.segments).map(([from, to]) => ({ from, to, color: STALL_COLOR })));
  const legend = bitrateSeries.map((s) => ({ label: s.label, color: s.color, dashed: s.dashed }));

  return (
    <>
      <section className="panel">
        <h2>Selected bitrate vs network bandwidth</h2>
        <p className="hint">Steps are the rung each algorithm requested. Bandwidth above {Math.round(bitrateMax)} kbps is clipped. Ladder rungs are drawn as faint guides.</p>
        <Legend items={legend} />
        <TimeChart series={bitrateSeries} xMax={xMax} yMax={bitrateMax} yLabel="kbps" guides={config.ladder_kbps} />
      </section>
      <section className="panel">
        <h2>Buffer level over time</h2>
        <p className="hint">Red bands are rebuffering stalls. The dashed guide is the {config.max_buffer_s}s buffer cap.</p>
        <Legend items={[...legend.slice(1), { label: "Stall", color: STALL_COLOR }]} />
        <TimeChart series={bufferSeries} xMax={xMax} yMax={config.max_buffer_s + 2} yLabel="seconds" bands={stalls} guides={[config.max_buffer_s]} />
      </section>
    </>
  );
}
