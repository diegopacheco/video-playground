import { useEffect, useState } from "react";
import { useSimulation } from "../hooks/useSimulation";
import type { AlgorithmInfo, TraceSummary } from "../types/abr";
import { ComparisonTable } from "./ComparisonTable";
import { Controls } from "./Controls";
import { SimulationCharts } from "./SimulationCharts";

interface Props {
  traces: TraceSummary[];
  algorithms: AlgorithmInfo[];
}

export function SimulateView({ traces, algorithms }: Props) {
  const [trace, setTrace] = useState(traces[0]?.id ?? "");
  const [selected, setSelected] = useState(algorithms.map((a) => a.id));
  const { report, running, error, run } = useSimulation();

  useEffect(() => {
    if (trace) void run(trace, selected);
  }, []);

  const toggle = (id: string) =>
    setSelected((current) => (current.includes(id) ? current.filter((x) => x !== id) : algorithms.map((a) => a.id).filter((x) => x === id || current.includes(x))));

  return (
    <>
      <Controls
        traces={traces}
        algorithms={algorithms}
        trace={trace}
        selected={selected}
        running={running}
        onTrace={setTrace}
        onToggle={toggle}
        onRun={() => void run(trace, selected)}
      />
      {error && <p className="error">{error}</p>}
      {report && (
        <>
          <section className="panel">
            <h2>Comparison on {report.trace.name}</h2>
            <p className="hint">
              {report.config.segments} segments of {report.config.segment_s}s, ladder {report.config.ladder_kbps.join(" / ")} kbps. QoE = mean per segment of bitrate (Mbps) minus switch magnitude minus {(report.config.ladder_kbps[report.config.ladder_kbps.length - 1] ?? 0) / 1000} x (rebuffer + startup seconds). Best value per column is highlighted.
            </p>
            <ComparisonTable runs={report.runs} />
          </section>
          <SimulationCharts report={report} />
        </>
      )}
    </>
  );
}
