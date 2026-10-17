import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import { MediaPlayer, type MediaPlayerClass, type QualityChangeRenderedEvent } from "dashjs";
import type { Level, PlayerState } from "../types/manifest";
import { switchMessage } from "../lib/format";

export function useDashPlayer(video: RefObject<HTMLVideoElement | null>, url: string): PlayerState {
  const [levels, setLevels] = useState<Level[]>([]);
  const [current, setCurrent] = useState<number | null>(null);
  const [selected, setSelected] = useState(-1);
  const [switches, setSwitches] = useState<string[]>([]);
  const playerRef = useRef<MediaPlayerClass | null>(null);

  useEffect(() => {
    const el = video.current;
    if (!el) return;
    const player = MediaPlayer().create();
    playerRef.current = player;
    let known: Level[] = [];
    player.on(MediaPlayer.events.STREAM_INITIALIZED, () => {
      known = player
        .getRepresentationsByType("video")
        .map((r) => ({ index: r.index, width: r.width, height: r.height, bitrate: r.bandwidth }));
      setLevels(known);
    });
    player.on(MediaPlayer.events.QUALITY_CHANGE_RENDERED, (e: QualityChangeRenderedEvent) => {
      if (e.mediaType !== "video") return;
      const from = e.oldRepresentation ? known[e.oldRepresentation.index] : undefined;
      const message = switchMessage(from, known[e.newRepresentation.index], el.currentTime);
      setCurrent(e.newRepresentation.index);
      setSwitches((s) => [message, ...s].slice(0, 8));
    });
    el.muted = true;
    player.initialize(el, url, true);
    return () => {
      player.destroy();
      playerRef.current = null;
    };
  }, [video, url]);

  const setLevel = useCallback((index: number) => {
    const player = playerRef.current;
    if (!player) return;
    player.updateSettings({ streaming: { abr: { autoSwitchBitrate: { video: index < 0 } } } });
    if (index >= 0) player.setRepresentationForTypeByIndex("video", index, true);
    setSelected(index);
  }, []);

  return { levels, current, selected, switches, setLevel };
}
