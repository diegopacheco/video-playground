import { useState } from "react";
import { LiveMetricsView } from "./components/LiveMetricsView";
import { ProbeView } from "./components/ProbeView";
import { useLiveMetrics } from "./hooks/useLiveMetrics";
import { formatPercent } from "./lib/format";

type Tab = "metrics" | "probe";

export function App() {
  const [tab, setTab] = useState<Tab>("metrics");
  const live = useLiveMetrics();
  return (
    <main>
      <header>
        <h1>CDN Edge Monitor</h1>
        <span className="subtitle">origin + LRU edge with coalescing and stale-while-revalidate</span>
        <span className={`status ${live.error ? "down" : "up"}`}>
          {live.error ? "edge down" : `live, hit ratio ${formatPercent(live.edge?.hit_ratio ?? 0)}`}
        </span>
      </header>
      <nav>
        <button type="button" className={tab === "metrics" ? "tab active" : "tab"} onClick={() => setTab("metrics")}>
          Live Metrics
        </button>
        <button type="button" className={tab === "probe" ? "tab active" : "tab"} onClick={() => setTab("probe")}>
          Coalescing Probe
        </button>
      </nav>
      {tab === "metrics" ? <LiveMetricsView live={live} /> : <ProbeView />}
    </main>
  );
}
