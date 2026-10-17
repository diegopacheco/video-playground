import type { RefObject } from "react";
import type { LogLevel } from "../types";

interface Props {
  videoRef: RefObject<HTMLVideoElement | null>;
  manifestUrl: string;
  status: LogLevel;
}

const LABEL: Record<LogLevel, string> = { info: "idle", ok: "playing", error: "blocked" };

export function PlayerView({ videoRef, manifestUrl, status }: Props) {
  return (
    <section className="card">
      <h2>
        Player <span className={`badge ${status}`}>{LABEL[status]}</span>
      </h2>
      <video ref={videoRef} className="video" muted controls playsInline />
      <div className="mono small">{manifestUrl || "no manifest yet"}</div>
    </section>
  );
}
