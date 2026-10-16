import { useEffect, useRef, useState, type RefObject } from "react";
import { correct, driftMs, expectedPosition, type Correction } from "../sync/drift";
import type { PlaybackState } from "../types/protocol";

const TICK_MS = 200;
const HEARTBEAT_MS = 1000;
const HISTORY = 75;

export interface SyncStats {
  drift: number;
  expected: number;
  actual: number;
  correction: Correction["kind"] | "leader";
  playbackRate: number;
  history: number[];
}

interface Options {
  video: RefObject<HTMLVideoElement | null>;
  state: PlaybackState | null;
  isLeader: boolean;
  serverNow: () => number;
  sendState: (state: PlaybackState) => void;
}

function snapshot(v: HTMLVideoElement, at: number): PlaybackState {
  return { playing: !v.paused, position: v.currentTime, rate: v.playbackRate, at };
}

function syncPlayPause(v: HTMLVideoElement, playing: boolean) {
  if (playing && v.paused) v.play().catch(() => undefined);
  if (!playing && !v.paused) v.pause();
}

function apply(v: HTMLVideoElement, c: Correction) {
  if (c.kind === "seek") v.currentTime = Math.min(c.position, Number.isFinite(v.duration) ? v.duration : c.position);
  if (Math.abs(v.playbackRate - c.rate) > 1e-3) v.playbackRate = c.rate;
}

export function useSync({ video, state, isLeader, serverNow, sendState }: Options): SyncStats {
  const stateRef = useRef(state);
  stateRef.current = state;
  const history = useRef<number[]>([]);
  const nudging = useRef(false);
  const [stats, setStats] = useState<SyncStats>({
    drift: 0,
    expected: 0,
    actual: 0,
    correction: "hold",
    playbackRate: 1,
    history: [],
  });

  useEffect(() => {
    const v = video.current;
    if (!v || !isLeader) return;
    const s = stateRef.current;
    if (s) v.playbackRate = s.rate;
    const publish = () => sendState(snapshot(v, serverNow()));
    const events = ["play", "pause", "seeked", "ratechange"] as const;
    events.forEach((e) => v.addEventListener(e, publish));
    const heartbeat = window.setInterval(publish, HEARTBEAT_MS);
    publish();
    return () => {
      events.forEach((e) => v.removeEventListener(e, publish));
      window.clearInterval(heartbeat);
    };
  }, [video, isLeader, serverNow, sendState]);

  useEffect(() => {
    const tick = () => {
      const v = video.current;
      const s = stateRef.current;
      if (!v || !s || v.readyState < 1) return;
      const expected = expectedPosition(s, serverNow());
      const drift = driftMs(v.currentTime, expected);
      let kind: SyncStats["correction"] = "leader";
      if (!isLeader) {
        syncPlayPause(v, s.playing);
        const c = correct(drift, s, expected, nudging.current);
        nudging.current = c.kind === "nudge";
        kind = c.kind;
        if (!v.seeking) apply(v, c);
      }
      history.current = [...history.current, drift].slice(-HISTORY);
      setStats({
        drift,
        expected,
        actual: v.currentTime,
        correction: kind,
        playbackRate: v.playbackRate,
        history: history.current,
      });
    };
    const id = window.setInterval(tick, TICK_MS);
    return () => window.clearInterval(id);
  }, [video, isLeader, serverNow]);

  return stats;
}
