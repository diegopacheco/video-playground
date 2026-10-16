import { useEffect, useState, type RefObject } from "react";
import { bufferAhead } from "../lib/format";

export function useBufferAhead(video: RefObject<HTMLVideoElement | null>): number {
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    const timer = window.setInterval(() => {
      const el = video.current;
      if (!el) return;
      const ranges = Array.from({ length: el.buffered.length }, (_, i) => ({ start: el.buffered.start(i), end: el.buffered.end(i) }));
      setSeconds(bufferAhead(ranges, el.currentTime));
    }, 500);
    return () => window.clearInterval(timer);
  }, [video]);
  return seconds;
}
