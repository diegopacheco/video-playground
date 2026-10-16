import { useSimulation } from "../hooks/useSimulation";
import { ComparisonTable } from "./ComparisonTable";
import { ConfigPanel } from "./ConfigPanel";
import { LoadChart } from "./LoadChart";

export function SimulatorView() {
  const { config, setConfig, reports, running, error, elapsedMs, run } = useSimulation();
  if (!config) return <p className="muted">{error ?? "Loading default scenario..."}</p>;
  return (
    <div className="stack">
      <ConfigPanel config={config} running={running} onChange={setConfig} onRun={run} />
      {error && <p className="error">{error}</p>}
      {elapsedMs !== null && (
        <p className="muted">
          Simulated {config.users.toLocaleString()} viewers x {reports.length} strategies in {elapsedMs} ms (seed {config.seed}).
        </p>
      )}
      {reports.length > 0 && (
        <>
          <LoadChart reports={reports} originCapacity={config.origin_rps} apiCapacity={config.api_rps} />
          <ComparisonTable reports={reports} />
        </>
      )}
    </div>
  );
}
