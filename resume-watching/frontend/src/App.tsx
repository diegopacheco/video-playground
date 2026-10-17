import { useCallback } from "react";
import { fetchRow, fetchStats, flushNow, resetProfile } from "./api/client";
import { ContinueRow } from "./components/ContinueRow";
import { DevicePanel } from "./components/DevicePanel";
import { StatsBar } from "./components/StatsBar";
import { useDevice } from "./hooks/useDevice";
import { usePoll } from "./hooks/usePoll";

const USER = "account-42";
const PROFILE = "profile-ana";
const HEARTBEAT_SECS = 2;
const SPEED = 5;

export function App() {
  const tv = useDevice({ deviceId: "living-room-tv", userId: USER, profileId: PROFILE, initialTitle: "dark-harbor", heartbeatSecs: HEARTBEAT_SECS, speed: SPEED });
  const phone = useDevice({ deviceId: "phone", userId: USER, profileId: PROFILE, initialTitle: "lunar-kitchen", heartbeatSecs: HEARTBEAT_SECS, speed: SPEED });
  const loadRow = useCallback(() => fetchRow(PROFILE), []);
  const row = usePoll(loadRow, 1000);
  const stats = usePoll(fetchStats, 1000);

  const refresh = () => {
    row.refresh();
    stats.refresh();
  };

  return (
    <main>
      <header className="top">
        <h1>Resume Watching</h1>
        <p>
          One profile (<code>{PROFILE}</code>), two devices. Players send a heartbeat every {HEARTBEAT_SECS}s at {SPEED}x speed ({HEARTBEAT_SECS * SPEED}s of content). The server
          coalesces heartbeats per profile and title, resolves conflicts by client timestamp, and flushes the latest position to the store.
        </p>
      </header>
      <StatsBar stats={stats.data} onFlush={() => void flushNow().then(refresh)} onReset={() => void resetProfile(PROFILE).then(refresh)} />
      {(row.error || stats.error) && <p className="error">{row.error ?? stats.error}</p>}
      <div className="devices">
        <DevicePanel label="Living room TV" deviceId="living-room-tv" device={tv} />
        <DevicePanel label="Phone" deviceId="phone" device={phone} />
      </div>
      <h2 className="row-title">Continue watching</h2>
      <ContinueRow items={row.data ?? []} />
    </main>
  );
}
