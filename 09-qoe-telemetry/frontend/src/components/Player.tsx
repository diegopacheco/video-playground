import Hls from "hls.js";
import { useEffect, useRef, useState } from "react";
import { postBeacons } from "../api/telemetry";
import { PlayerTracker } from "../lib/playerTracker";
import type { Beacon } from "../types/telemetry";

const SOURCE = "/media/clip/master.m3u8";
const TITLE = "local-clip";
const CDN = "local-origin";
const HEARTBEAT_MS = 2000;

interface LogLine {
  beacon: Beacon;
  status: "sent" | "failed";
}

export function Player() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const [sessionId] = useState(() => crypto.randomUUID().slice(0, 8));
  const [log, setLog] = useState<LogLine[]>([]);
  const [levels, setLevels] = useState<number[]>([]);
  const [level, setLevel] = useState(-1);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const tracker = new PlayerTracker(`player-${sessionId}`, TITLE, CDN);
    const emit = (beacons: Beacon[]) => {
      if (beacons.length === 0) return;
      postBeacons(beacons)
        .then(() => record(beacons, "sent"))
        .catch(() => record(beacons, "failed"));
    };
    const record = (beacons: Beacon[], status: LogLine["status"]) =>
      setLog((cur) => [...beacons.map((beacon) => ({ beacon, status })).reverse(), ...cur].slice(0, 40));

    const hls = new Hls({ startLevel: 0 });
    hlsRef.current = hls;
    hls.loadSource(SOURCE);
    hls.attachMedia(video);
    hls.on(Hls.Events.MANIFEST_PARSED, () => setLevels(hls.levels.map((l) => Math.round(l.bitrate / 1000))));
    hls.on(Hls.Events.LEVEL_SWITCHED, (_, data) => {
      const kbps = Math.round((hls.levels[data.level]?.bitrate ?? 0) / 1000);
      emit(tracker.levelSwitched(Date.now(), kbps));
    });
    hls.on(Hls.Events.ERROR, (_, data) => {
      if (data.fatal) emit(tracker.error(Date.now(), data.details));
    });

    const onPlay = () => emit(tracker.requestPlay(Date.now()));
    const onPlaying = () => emit(tracker.playing(Date.now()));
    const onWaiting = () => emit(tracker.waiting(Date.now()));
    const onError = () => emit(tracker.error(Date.now(), `MEDIA_ERR_${video.error?.code ?? 0}`));
    video.addEventListener("play", onPlay);
    video.addEventListener("playing", onPlaying);
    video.addEventListener("waiting", onWaiting);
    video.addEventListener("error", onError);
    const beat = window.setInterval(() => emit(tracker.heartbeat(Date.now(), video.paused || video.ended)), HEARTBEAT_MS);

    return () => {
      window.clearInterval(beat);
      video.removeEventListener("play", onPlay);
      video.removeEventListener("playing", onPlaying);
      video.removeEventListener("waiting", onWaiting);
      video.removeEventListener("error", onError);
      hls.destroy();
      hlsRef.current = null;
    };
  }, [sessionId]);

  const pickLevel = (next: number) => {
    setLevel(next);
    if (hlsRef.current) hlsRef.current.currentLevel = next;
  };

  return (
    <div className="player-grid">
      <section className="card">
        <header className="card-head">
          <h2>Real player</h2>
          <span className="muted">
            session player-{sessionId} · title {TITLE} · cdn {CDN}
          </span>
        </header>
        <video ref={videoRef} controls muted loop playsInline className="video" />
        <div className="levels">
          <span className="muted">Rendition</span>
          <button className={level === -1 ? "on" : ""} onClick={() => pickLevel(-1)}>
            auto
          </button>
          {levels.map((kbps, i) => (
            <button key={kbps} className={level === i ? "on" : ""} onClick={() => pickLevel(i)}>
              {kbps} kbps
            </button>
          ))}
        </div>
      </section>
      <section className="card">
        <header className="card-head">
          <h2>Beacons sent by this player</h2>
          <span className="muted">POST /api/beacons, newest first</span>
        </header>
        <ul className="log">
          {log.length === 0 && <li className="muted">press play to start the session</li>}
          {log.map((l, i) => (
            <li key={`${l.beacon.ts}-${i}`} className={l.status === "failed" ? "bad-text" : ""}>
              <code>{new Date(l.beacon.ts).toLocaleTimeString()}</code> <strong>{l.beacon.kind}</strong> {describe(l.beacon)}
              {l.status === "failed" && " (failed)"}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

function describe(b: Beacon): string {
  switch (b.kind) {
    case "startup":
      return `${b.startup_ms} ms`;
    case "rebuffer_end":
      return `${b.rebuffer_ms} ms stalled`;
    case "bitrate_switch":
      return `${b.from_kbps} -> ${b.to_kbps} kbps`;
    case "error":
      return b.code;
    case "heartbeat":
      return `${b.played_ms} ms played`;
    case "rebuffer_start":
      return "";
  }
}
