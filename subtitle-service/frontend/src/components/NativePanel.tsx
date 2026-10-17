import { useEffect, useRef, useState } from "react";
import { SOURCE_URL, fetchVtt, vttUrl } from "../api/client";
import { extractCueStyles, formatBytes, formatPercent } from "../lib/format";
import type { TrackInfo } from "../types";
import { Panel } from "./Panel";

interface Props {
  track: TrackInfo;
  videoRef: (el: HTMLVideoElement | null) => void;
}

export function NativePanel({ track, videoRef }: Props) {
  const local = useRef<HTMLVideoElement | null>(null);
  const [css, setCss] = useState("");

  useEffect(() => {
    let live = true;
    fetchVtt(track.locale).then(
      (vtt) => live && setCss(extractCueStyles(vtt).replaceAll("::cue(", ".native-video::cue(")),
      () => live && setCss(""),
    );
    const video = local.current;
    const show = () => {
      if (!video) return;
      for (const t of Array.from(video.textTracks)) t.mode = t.language === track.locale ? "showing" : "disabled";
    };
    show();
    video?.textTracks.addEventListener("addtrack", show);
    return () => {
      live = false;
      video?.textTracks.removeEventListener("addtrack", show);
    };
  }, [track.locale]);

  return (
    <Panel
      title="Native WebVTT"
      subtitle="<track> rendered by the browser from the converted VTT"
      footer={
        <span>
          {formatBytes(track.vttBytes)} VTT, fidelity {formatPercent(track.report.fidelity)}, server convert{" "}
          {track.convertMicros} us
        </span>
      }
    >
      <style>{css}</style>
      <video
        className="native-video"
        ref={(el) => {
          local.current = el;
          videoRef(el);
        }}
        src={SOURCE_URL}
        muted
        playsInline
        loop
        preload="auto"
      >
        <track key={track.locale} kind="subtitles" src={vttUrl(track.locale)} srcLang={track.locale} label={track.name} default />
      </video>
    </Panel>
  );
}
