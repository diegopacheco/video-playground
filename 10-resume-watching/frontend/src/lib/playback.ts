import type { Title } from "../types";

export const FINISHED_RATIO = 0.9;

export const TITLES: Title[] = [
  { id: "dark-harbor", name: "Dark Harbor", episode: "s1e3", duration: 600, hue: 210 },
  { id: "lunar-kitchen", name: "Lunar Kitchen", episode: "s2e1", duration: 420, hue: 30 },
  { id: "the-long-relay", name: "The Long Relay", episode: "movie", duration: 900, hue: 140 },
  { id: "glass-city", name: "Glass City", episode: "s1e1", duration: 480, hue: 300 },
];

export function findTitle(id: string): Title {
  return TITLES.find((t) => t.id === id) ?? (TITLES[0] as Title);
}

export function advance(position: number, step: number, duration: number): number {
  return Math.min(duration, Math.max(0, position + step));
}

export function isFinished(position: number, duration: number): boolean {
  return duration > 0 && position / duration > FINISHED_RATIO;
}

export function formatTime(secs: number): string {
  const s = Math.max(0, Math.floor(secs));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, "0")}`;
}

export function heartbeatDue(tick: number, everySecs: number): boolean {
  return everySecs > 0 && tick % everySecs === 0;
}
