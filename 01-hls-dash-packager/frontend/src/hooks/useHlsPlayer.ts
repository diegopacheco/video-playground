import { useCallback, useEffect, useRef, useState, type RefObject } from "react";
import Hls from "hls.js";
import type { Level, PlayerState } from "../types/manifest";
import { switchMessage } from "../lib/format";

export function useHlsPlayer(video: RefObject<HTMLVideoElement | null>, url: string): PlayerState {
  const [levels, setLevels] = useState<Level[]>([]);
  const [current, setCurrent] = useState<number | null>(null);
  const [selected, setSelected] = useState(-1);
  const [switches, setSwitches] = useState<string[]>([]);
  const hlsRef = useRef<Hls | null>(null);

  useEffect(() => {
    const el = video.current;
    if (!el || !Hls.isSupported()) return;
    const hls = new Hls();
    hlsRef.current = hls;
    let known: Level[] = [];
    let last: number | null = null;
    hls.on(Hls.Events.MANIFEST_PARSED, (_, data) => {
      known = data.levels.map((l, index) => ({ index, width: l.width, height: l.height, bitrate: l.bitrate }));
      setLevels(known);
    });
    hls.on(Hls.Events.LEVEL_SWITCHED, (_, data) => {
      const message = switchMessage(last === null ? undefined : known[last], known[data.level], el.currentTime);
      last = data.level;
      setCurrent(data.level);
      setSwitches((s) => [message, ...s].slice(0, 8));
    });
    hls.loadSource(url);
    hls.attachMedia(el);
    el.muted = true;
    void el.play().catch(() => undefined);
    return () => {
      hls.destroy();
      hlsRef.current = null;
    };
  }, [video, url]);

  const setLevel = useCallback((index: number) => {
    if (hlsRef.current) hlsRef.current.currentLevel = index;
    setSelected(index);
  }, []);

  return { levels, current, selected, switches, setLevel };
}
