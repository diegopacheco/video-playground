import type { RefObject } from "react";
import type { PlayerState } from "../types/manifest";
import { levelLabel } from "../lib/format";
import { useBufferAhead } from "../hooks/useBufferAhead";

interface Props {
  title: string;
  url: string;
  segmentSeconds: number;
  video: RefObject<HTMLVideoElement | null>;
  state: PlayerState;
}

export function PlayerPanel({ title, url, segmentSeconds, video, state }: Props) {
  const buffer = useBufferAhead(video);
  const current = state.current === null ? undefined : state.levels[state.current];
  return (
    <section className="player">
      <div className="video-wrap">
        <video ref={video} controls playsInline muted />
      </div>
      <aside className="card side">
        <h2>{title}</h2>
        <p className="mono small">{url}</p>
        <dl>
          <dt>Current rendition</dt>
          <dd data-testid="current-rendition">{levelLabel(current)}</dd>
          <dt>Mode</dt>
          <dd>{state.selected < 0 ? "Auto (ABR)" : "Forced"}</dd>
          <dt>Buffer ahead</dt>
          <dd>{buffer.toFixed(1)}s ({Math.ceil(buffer / segmentSeconds)} segments)</dd>
        </dl>
        <h3>Force level</h3>
        <div className="levels">
          <button className={state.selected < 0 ? "active" : ""} onClick={() => state.setLevel(-1)}>
            Auto
          </button>
          {state.levels.map((l) => (
            <button key={l.index} className={state.selected === l.index ? "active" : ""} onClick={() => state.setLevel(l.index)}>
              {l.height}p
            </button>
          ))}
        </div>
        <h3>Switch log</h3>
        <ul className="log mono small">
          {state.switches.length === 0 ? <li>waiting for first level</li> : state.switches.map((s, i) => <li key={i}>{s}</li>)}
        </ul>
      </aside>
    </section>
  );
}
