import { useCallback, useEffect, useRef, useState } from "react";
import { sendHeartbeat } from "../api/client";
import { advance, findTitle, heartbeatDue } from "../lib/playback";
import type { DeviceState } from "../types";

export interface DeviceConfig {
  deviceId: string;
  userId: string;
  profileId: string;
  initialTitle: string;
  heartbeatSecs: number;
  speed: number;
}

export interface Device {
  state: DeviceState;
  togglePlay: () => void;
  seek: (delta: number) => void;
  selectTitle: (titleId: string) => void;
  setSkew: (skewMs: number) => void;
}

export function useDevice(cfg: DeviceConfig): Device {
  const [state, setState] = useState<DeviceState>({
    titleId: cfg.initialTitle,
    position: 0,
    playing: false,
    skewMs: 0,
    sent: 0,
    lastResult: null,
  });
  const latest = useRef(state);
  const tick = useRef(0);

  useEffect(() => {
    latest.current = state;
  }, [state]);

  const beat = useCallback(
    async (titleId: string, position: number, skewMs: number) => {
      const title = findTitle(titleId);
      try {
        const result = await sendHeartbeat({
          user_id: cfg.userId,
          profile_id: cfg.profileId,
          device_id: cfg.deviceId,
          title_id: title.id,
          episode_id: title.episode,
          position_secs: Math.round(position),
          duration_secs: title.duration,
          client_ts_ms: Date.now() + skewMs,
        });
        setState((prev) => ({ ...prev, sent: prev.sent + 1, lastResult: result }));
      } catch {
        setState((prev) => ({ ...prev, lastResult: "error" }));
      }
    },
    [cfg.userId, cfg.profileId, cfg.deviceId],
  );

  useEffect(() => {
    if (!state.playing) return;
    const id = setInterval(() => {
      tick.current += 1;
      const s = latest.current;
      const duration = findTitle(s.titleId).duration;
      const position = advance(s.position, cfg.speed, duration);
      const playing = position < duration;
      latest.current = { ...s, position, playing };
      setState((prev) => ({ ...prev, position, playing }));
      if (heartbeatDue(tick.current, cfg.heartbeatSecs) || !playing) {
        void beat(s.titleId, position, s.skewMs);
      }
    }, 1000);
    return () => clearInterval(id);
  }, [state.playing, cfg.speed, cfg.heartbeatSecs, beat]);

  const togglePlay = useCallback(() => {
    const s = latest.current;
    if (s.playing) void beat(s.titleId, s.position, s.skewMs);
    setState((prev) => ({ ...prev, playing: !prev.playing }));
  }, [beat]);

  const seek = useCallback(
    (delta: number) => {
      const s = latest.current;
      const position = advance(s.position, delta, findTitle(s.titleId).duration);
      latest.current = { ...s, position };
      setState((prev) => ({ ...prev, position }));
      void beat(s.titleId, position, s.skewMs);
    },
    [beat],
  );

  const selectTitle = useCallback((titleId: string) => {
    setState((prev) => ({ ...prev, titleId, position: 0, playing: false }));
  }, []);

  const setSkew = useCallback((skewMs: number) => {
    setState((prev) => ({ ...prev, skewMs }));
  }, []);

  return { state, togglePlay, seek, selectTitle, setSkew };
}
