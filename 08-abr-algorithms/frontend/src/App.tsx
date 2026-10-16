import { useState } from "react";
import { SimulateView } from "./components/SimulateView";
import { TracesView } from "./components/TracesView";
import { useCatalog } from "./hooks/useCatalog";

type Tab = "simulate" | "traces";

export function App() {
  const { traces, algorithms, error } = useCatalog();
  const [tab, setTab] = useState<Tab>("simulate");
  const ready = traces.length > 0 && algorithms.length > 0;

  return (
    <main>
      <header>
        <h1>ABR Lab</h1>
        <p>Throughput, BOLA and Dynamic adaptive bitrate algorithms replayed over bundled network traces.</p>
        <nav>
          <button type="button" className={tab === "simulate" ? "tab active" : "tab"} onClick={() => setTab("simulate")}>
            Simulate
          </button>
          <button type="button" className={tab === "traces" ? "tab active" : "tab"} onClick={() => setTab("traces")}>
            Traces
          </button>
        </nav>
      </header>
      {error && <p className="error">{error}</p>}
      {ready && tab === "simulate" && <SimulateView traces={traces} algorithms={algorithms} />}
      {ready && tab === "traces" && <TracesView traces={traces} />}
    </main>
  );
}
