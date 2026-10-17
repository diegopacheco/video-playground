import { useRef } from "react";
import { useParty } from "../hooks/useParty";
import { useSync } from "../hooks/useSync";
import { clockSkewMs } from "../sync/localClock";
import { Members } from "./Members";
import { Stats } from "./Stats";

interface Props {
  room: string;
  name: string;
  onLeave: () => void;
}

export function Party({ room, name, onLeave }: Props) {
  const video = useRef<HTMLVideoElement>(null);
  const party = useParty(room, name);
  const isLeader = party.view !== null && party.view.leader === party.you;
  const sync = useSync({
    video,
    state: party.view?.state ?? null,
    isLeader,
    serverNow: party.serverNow,
    sendState: party.sendState,
  });
  const leaderName = party.view?.members.find((m) => m.id === party.view?.leader)?.name ?? "-";

  return (
    <div className="party">
      <header className="bar">
        <div>
          <strong>{room}</strong>
          <span className="muted"> as {name}</span>
        </div>
        <div className="bar-right">
          <span className={`badge ${isLeader ? "leader" : "follower"}`} data-testid="role">
            {isLeader ? "leader" : `following ${leaderName}`}
          </span>
          <span className={`dot ${party.connected ? "on" : "off"}`} />
          <button className="ghost" onClick={onLeave}>
            Leave
          </button>
        </div>
      </header>
      {party.error && <p className="error">{party.error}</p>}
      <div className="grid">
        <section className="card player">
          <video ref={video} src="/media/clip.mp4" muted playsInline preload="auto" controls={isLeader} />
          <p className="muted small">
            {isLeader
              ? "You are the leader: play, pause and seek with the player controls."
              : "Controls are locked: playback follows the leader."}
          </p>
        </section>
        <aside className="side">
          <Stats clock={party.clock} samples={party.samples} sync={sync} skewMs={clockSkewMs} />
          {party.view && <Members members={party.view.members} leader={party.view.leader} you={party.you} />}
        </aside>
      </div>
    </div>
  );
}
