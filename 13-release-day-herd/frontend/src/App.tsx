import { useState } from "react";
import { SimulatorView } from "./components/SimulatorView";
import { WaitingRoomView } from "./components/WaitingRoomView";

type Tab = "simulator" | "room";

export function App() {
  const [tab, setTab] = useState<Tab>("simulator");
  return (
    <main>
      <header>
        <h1>Release Day Herd</h1>
        <p className="muted">One million viewers press play at 00:00. Which launch strategy survives?</p>
        <nav>
          <button className={tab === "simulator" ? "tab active" : "tab"} onClick={() => setTab("simulator")}>
            Simulator
          </button>
          <button className={tab === "room" ? "tab active" : "tab"} onClick={() => setTab("room")}>
            Live waiting room
          </button>
        </nav>
      </header>
      {tab === "simulator" ? <SimulatorView /> : <WaitingRoomView />}
    </main>
  );
}
