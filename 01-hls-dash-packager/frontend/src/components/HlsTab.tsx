import { useRef } from "react";
import { useHlsPlayer } from "../hooks/useHlsPlayer";
import { PlayerPanel } from "./PlayerPanel";

export function HlsTab({ url, segmentSeconds }: { url: string; segmentSeconds: number }) {
  const video = useRef<HTMLVideoElement | null>(null);
  const state = useHlsPlayer(video, url);
  return <PlayerPanel title="HLS via hls.js" url={url} segmentSeconds={segmentSeconds} video={video} state={state} />;
}
