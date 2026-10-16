import type { PlaybackState } from "../types/protocol";

export const DEADBAND_MS = 40;
export const SETTLE_MS = 10;
export const SEEK_THRESHOLD_MS = 1000;
export const PAUSED_SEEK_MS = 50;
export const MAX_NUDGE = 0.05;
export const GAIN_PER_SECOND = 0.5;

export type Correction =
  | { kind: "hold"; rate: number }
  | { kind: "nudge"; rate: number }
  | { kind: "seek"; rate: number; position: number };

export function expectedPosition(state: PlaybackState, serverNow: number): number {
  if (!state.playing) return state.position;
  const elapsedS = Math.max(0, serverNow - state.at) / 1000;
  return state.position + elapsedS * state.rate;
}

export function driftMs(actualS: number, expectedS: number): number {
  return (actualS - expectedS) * 1000;
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, v));
}

export function correct(drift: number, state: PlaybackState, expected: number, nudging = false): Correction {
  const abs = Math.abs(drift);
  if (!state.playing) {
    return abs > PAUSED_SEEK_MS ? { kind: "seek", rate: state.rate, position: expected } : { kind: "hold", rate: state.rate };
  }
  if (abs >= SEEK_THRESHOLD_MS) return { kind: "seek", rate: state.rate, position: expected };
  if (abs <= (nudging ? SETTLE_MS : DEADBAND_MS)) return { kind: "hold", rate: state.rate };
  const factor = clamp(1 - (drift / 1000) * GAIN_PER_SECOND, 1 - MAX_NUDGE, 1 + MAX_NUDGE);
  return { kind: "nudge", rate: state.rate * factor };
}
