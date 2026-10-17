import Hls from "hls.js";
import { useEffect, useState, type RefObject } from "react";
import { playerConfig, playlistUrl } from "../api/streams";
import { appendBounded, latencyMs } from "../lib/stats";
import type { History, Mode } from "../types/latency";

const SAMPLE_EVERY_MS = 250;
const HISTORY_LIMIT = 480;

export function useLatencyPlayer(video: RefObject<HTMLVideoElement | null>, mode: Mode) {
  const [history, setHistory] = useState<History>({ ll: [], regular: [] });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const element = video.current;
    if (!element) return;
    if (!Hls.isSupported()) {
      setError("Media Source Extensions are not available in this browser");
      return;
    }
    setError(null);
    const hls = new Hls(playerConfig(mode));
    hls.on(Hls.Events.ERROR, (_event, data) => {
      if (data.fatal) setError(`${data.type}: ${data.details}`);
    });
    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      element.play().catch(() => setError("autoplay blocked, press play"));
    });
    hls.loadSource(playlistUrl(mode));
    hls.attachMedia(element);
    const timer = window.setInterval(() => {
      if (element.paused) return;
      const now = Date.now();
      const value = latencyMs(now, hls.playingDate);
      if (value === null) return;
      setHistory((prev) => ({
        ...prev,
        [mode]: appendBounded(prev[mode], { at: now, latencyMs: value }, HISTORY_LIMIT),
      }));
    }, SAMPLE_EVERY_MS);
    return () => {
      window.clearInterval(timer);
      hls.destroy();
    };
  }, [video, mode]);

  return { history, error };
}
