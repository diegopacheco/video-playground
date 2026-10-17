import { useState } from "react";
import { AuditPanel } from "./components/AuditPanel";
import { EventLog } from "./components/EventLog";
import { LicensePanel } from "./components/LicensePanel";
import { PlayerView } from "./components/PlayerView";
import { ScenarioPanel } from "./components/ScenarioPanel";
import { useAudit } from "./hooks/useAudit";
import { useDrmPlayer } from "./hooks/useDrmPlayer";
import type { Scenario } from "./types";

export function App() {
  const { videoRef, run, logs, session, manifestUrl, license, status } = useDrmPlayer();
  const audit = useAudit(1500);
  const [active, setActive] = useState("");

  const onRun = (s: Scenario) => {
    setActive(s.id);
    void run(s);
  };

  return (
    <main>
      <header>
        <h1>DRM License Server</h1>
        <p>CENC (cenc, AES-CTR) DASH, W3C Clear Key license exchange, HMAC session tokens and signed segment URLs.</p>
      </header>
      <div className="layout">
        <div className="col">
          <ScenarioPanel active={active} onRun={onRun} />
          <EventLog logs={logs} />
        </div>
        <div className="col wide">
          <PlayerView videoRef={videoRef} manifestUrl={manifestUrl} status={status} />
          <LicensePanel session={session} license={license} />
          <AuditPanel events={audit} />
        </div>
      </div>
    </main>
  );
}
