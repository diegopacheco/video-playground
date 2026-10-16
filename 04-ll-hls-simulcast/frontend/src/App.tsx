import { useEffect, useRef, useState } from "react";
import { MODE_LABEL, playlistUrl } from "./api/streams";
import { LatencyChart } from "./components/LatencyChart";
import { ModeToggle } from "./components/ModeToggle";
import { StatsPanel } from "./components/StatsPanel";
import { WallClock } from "./components/WallClock";
import { useLatencyPlayer } from "./hooks/useLatencyPlayer";
import { summarize } from "./lib/stats";
import type { Mode, Summary } from "./types/latency";

declare global {
  interface Window {
    latencySummary?: Record<Mode, Summary>;
  }
}

export function App() {
  const [mode, setMode] = useState<Mode>("ll");
  const video = useRef<HTMLVideoElement>(null);
  const { history, error } = useLatencyPlayer(video, mode);

  useEffect(() => {
    window.latencySummary = { ll: summarize(history.ll), regular: summarize(history.regular) };
  }, [history]);

  return (
    <main>
      <header>
        <h1>LL-HLS Latency Lab</h1>
        <p>
          One live encode, two playlists. The Rust origin publishes 200 ms fMP4 parts in an LL-HLS playlist and whole
          2 s segments in a regular one. Latency = browser clock minus the playhead's EXT-X-PROGRAM-DATE-TIME.
        </p>
      </header>
      <ModeToggle mode={mode} onChange={setMode} />
      <section className="stage">
        <div className="player">
          <video ref={video} muted playsInline controls />
          <div className="meta">
            <span>
              Playing <strong>{MODE_LABEL[mode]}</strong> from <code>{playlistUrl(mode)}</code>
            </span>
            {error && <span className="error">{error}</span>}
          </div>
          <WallClock />
        </div>
        <div className="panel">
          <h2>Glass-to-glass latency</h2>
          <LatencyChart history={history} />
          <div className="legend">
            <span><span className="swatch ll" /> {MODE_LABEL.ll}</span>
            <span><span className="swatch regular" /> {MODE_LABEL.regular}</span>
          </div>
          <StatsPanel history={history} active={mode} />
        </div>
      </section>
    </main>
  );
}
