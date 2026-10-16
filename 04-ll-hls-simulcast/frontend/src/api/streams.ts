import type { HlsConfig } from "hls.js";
import type { Mode } from "../types/latency";

export const MODES: Mode[] = ["ll", "regular"];

export const MODE_LABEL: Record<Mode, string> = {
  ll: "Low-Latency HLS",
  regular: "Regular HLS",
};

export function playlistUrl(mode: Mode): string {
  return mode === "ll" ? "/live/ll.m3u8" : "/live/hls.m3u8";
}

export function playerConfig(mode: Mode): Partial<HlsConfig> {
  return mode === "ll"
    ? { lowLatencyMode: true, backBufferLength: 10 }
    : { lowLatencyMode: false, backBufferLength: 10 };
}
