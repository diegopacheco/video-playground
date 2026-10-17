import { useState } from "react";
import { Dashboard } from "./components/Dashboard";
import { Player } from "./components/Player";
import { useWindows } from "./hooks/useWindows";

type Tab = "dashboard" | "player";

export function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const windows = useWindows();
  return (
    <main className="app">
      <header className="top">
        <div>
          <h1>QoE Telemetry</h1>
          <p className="muted">Player beacons → Kafka → Rust tumbling 10s windows → SSE</p>
        </div>
        <nav className="tabs">
          <button className={tab === "dashboard" ? "on" : ""} onClick={() => setTab("dashboard")}>
            Live dashboard
          </button>
          <button className={tab === "player" ? "on" : ""} onClick={() => setTab("player")}>
            Player
          </button>
        </nav>
      </header>
      {tab === "dashboard" ? <Dashboard state={windows} /> : <Player />}
    </main>
  );
}
