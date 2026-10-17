import { useRef } from "react";
import { useDashPlayer } from "../hooks/useDashPlayer";
import { PlayerPanel } from "./PlayerPanel";

export function DashTab({ url, segmentSeconds }: { url: string; segmentSeconds: number }) {
  const video = useRef<HTMLVideoElement | null>(null);
  const state = useDashPlayer(video, url);
  return <PlayerPanel title="DASH via dash.js" url={url} segmentSeconds={segmentSeconds} video={video} state={state} />;
}
