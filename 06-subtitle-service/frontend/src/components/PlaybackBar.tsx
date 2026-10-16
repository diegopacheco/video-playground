import type { SyncedPlayback } from "../hooks/useSyncedPlayback";

export function PlaybackBar({ playback }: { playback: SyncedPlayback }) {
  const { playing, time, duration, toggle, seek } = playback;
  return (
    <div className="playback">
      <button className="primary" onClick={toggle}>
        {playing ? "Pause all" : "Play all"}
      </button>
      <input
        type="range"
        min={0}
        max={duration || 16}
        step={0.1}
        value={time}
        onChange={(e) => seek(Number(e.target.value))}
        aria-label="Seek"
      />
      <span className="clock">
        {time.toFixed(1)}s / {(duration || 0).toFixed(1)}s
      </span>
    </div>
  );
}
