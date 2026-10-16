import { useEffect, useState } from "react";
import { fetchUsers, manifestUrl } from "./api/client";
import { BeaconLog } from "./components/BeaconLog";
import { BreakSummary } from "./components/BreakSummary";
import { ManifestPanel } from "./components/ManifestPanel";
import { Player } from "./components/Player";
import { UserPicker } from "./components/UserPicker";
import { useBeacons } from "./hooks/useBeacons";
import { useStitch } from "./hooks/useStitch";
import { sessionFor } from "./manifest";
import type { UserProfile } from "./types";

function initialUser(): string {
  return new URLSearchParams(window.location.search).get("user") ?? "alice";
}

export function App() {
  const [users, setUsers] = useState<UserProfile[]>([]);
  const [user, setUser] = useState(initialUser);
  const [session, setSession] = useState(() => sessionFor(initialUser(), Date.now()));
  const { view, error } = useStitch(user, session);
  const beacons = useBeacons(session);

  useEffect(() => {
    fetchUsers().then(setUsers).catch(() => undefined);
  }, []);

  const select = (id: string) => {
    setUser(id);
    setSession(sessionFor(id, Date.now()));
  };

  return (
    <main>
      <header className="top">
        <div>
          <h1>SSAI Stitcher</h1>
          <p>Server-side ad insertion: SCTE-35 breaks replaced per user, discontinuities around every creative, beacons recorded on segment fetch.</p>
        </div>
        <UserPicker users={users} selected={user} onSelect={select} />
      </header>
      {error && <div className="error">{error}</div>}
      <div className="grid">
        <div className="left">
          <Player src={manifestUrl(user, session)} />
          <div className="session">session {session}</div>
          {view && <BreakSummary breaks={view.breaks} />}
        </div>
        <BeaconLog beacons={beacons} />
      </div>
      {view && (
        <div className="side">
          <ManifestPanel title="Raw content playlist" manifest={view.raw} />
          <ManifestPanel title={`Stitched for ${view.user.name}`} manifest={view.stitched} />
        </div>
      )}
    </main>
  );
}
