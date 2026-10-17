import { useCallback, useEffect, useRef, useState } from "react";

const DRIFT_SECONDS = 0.12;

export interface SyncedPlayback {
  register: (index: number) => (el: HTMLVideoElement | null) => void;
  playing: boolean;
  time: number;
  duration: number;
  toggle: () => void;
  seek: (t: number) => void;
}

export function useSyncedPlayback(): SyncedPlayback {
  const videos = useRef<(HTMLVideoElement | null)[]>([]);
  const [playing, setPlaying] = useState(false);
  const [time, setTime] = useState(0);
  const [duration, setDuration] = useState(0);

  const all = (): HTMLVideoElement[] => videos.current.filter((v): v is HTMLVideoElement => v !== null);

  const register = useCallback(
    (index: number) => (el: HTMLVideoElement | null) => {
      videos.current[index] = el;
    },
    [],
  );

  const seek = useCallback((t: number) => {
    for (const v of all()) v.currentTime = t;
    setTime(t);
  }, []);

  const toggle = useCallback(() => {
    const list = all();
    const leader = list[0];
    if (!leader) return;
    if (leader.paused) {
      for (const v of list) {
        v.currentTime = leader.currentTime;
        void v.play().catch(() => undefined);
      }
      setPlaying(true);
    } else {
      for (const v of list) v.pause();
      setPlaying(false);
    }
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => {
      const [leader, ...followers] = all();
      if (!leader) return;
      setTime(leader.currentTime);
      if (Number.isFinite(leader.duration)) setDuration(leader.duration);
      for (const v of followers) {
        if (Math.abs(v.currentTime - leader.currentTime) > DRIFT_SECONDS) v.currentTime = leader.currentTime;
        if (!leader.paused && v.paused) void v.play().catch(() => undefined);
      }
    }, 250);
    return () => window.clearInterval(id);
  }, []);

  return { register, playing, time, duration, toggle, seek };
}
