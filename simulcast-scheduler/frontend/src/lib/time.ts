import type { Board, Entitlement } from "../types";

export function formatCountdown(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const clock = [hours, minutes, seconds].map((n) => String(n).padStart(2, "0")).join(":");
  return days > 0 ? `${days}d ${clock}` : clock;
}

export function formatOffset(ms: number): string {
  if (ms === 0) return "real time";
  const sign = ms > 0 ? "+" : "-";
  return `${sign}${formatCountdown(Math.abs(ms))}`;
}

export function nextUnlockMs(board: Board): number | null {
  const pending = board.episodes
    .flatMap((e) => e.regions)
    .map((r) => r.countdown_ms)
    .filter((ms): ms is number => ms !== null && ms > 0);
  return pending.length === 0 ? null : Math.min(...pending);
}

export function cellText(region: Entitlement): string {
  if (region.status === "available") return "OPEN";
  if (region.status === "locked") return formatCountdown(region.countdown_ms ?? 0);
  return region.reason === "license_expired" ? "EXPIRED" : "BLOCKED";
}
