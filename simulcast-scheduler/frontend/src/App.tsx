import { useCallback, useState } from "react";
import { api } from "./api/client";
import { ClockControl } from "./components/ClockControl";
import { CountdownView } from "./components/CountdownView";
import { HerdPanel } from "./components/HerdPanel";
import { StatusBoard } from "./components/StatusBoard";
import { usePoll } from "./hooks/usePoll";

const tabs = ["Countdowns", "Status Board", "Herd Metrics"] as const;
type Tab = (typeof tabs)[number];

export function App() {
  const [tab, setTab] = useState<Tab>("Countdowns");
  const loadBoard = useCallback(() => api.board(), []);
  const { data: board, error, refresh } = usePoll(loadBoard, 500);

  return (
    <main>
      <header>
        <h1>Simulcast Release Scheduler</h1>
        <nav>
          {tabs.map((t) => (
            <button key={t} className={t === tab ? "tab active" : "tab"} onClick={() => setTab(t)}>
              {t}
            </button>
          ))}
        </nav>
      </header>
      {error && <p className="error">{error}</p>}
      {board && (
        <>
          <ClockControl board={board} onChange={refresh} />
          {tab === "Countdowns" && <CountdownView board={board} onChange={refresh} />}
          {tab === "Status Board" && <StatusBoard board={board} />}
          {tab === "Herd Metrics" && <HerdPanel board={board} />}
        </>
      )}
    </main>
  );
}
