import { burnedUrl } from "../api/client";
import type { TrackInfo } from "../types";
import { Panel } from "./Panel";

interface Props {
  track: TrackInfo;
  videoRef: (el: HTMLVideoElement | null) => void;
}

export function BurnedPanel({ track, videoRef }: Props) {
  return (
    <Panel
      title="Burned-in"
      subtitle="ffmpeg ass filter baked pixels into a per-locale encode"
      footer={<span>one full video file per locale, zero client work</span>}
    >
      <video ref={videoRef} src={burnedUrl(track.locale)} muted playsInline loop preload="auto" />
    </Panel>
  );
}
