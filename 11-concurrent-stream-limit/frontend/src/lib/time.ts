export function remainingMs(expiresAt: number, serverNow: number, fetchedAt: number, localNow: number): number {
  const estimatedServerNow = serverNow + (localNow - fetchedAt);
  return Math.max(0, expiresAt - estimatedServerNow);
}

export function fraction(remaining: number, ttl: number): number {
  if (ttl <= 0) {
    return 0;
  }
  return Math.min(1, Math.max(0, remaining / ttl));
}

export function seconds(ms: number): string {
  return `${(ms / 1000).toFixed(1)}s`;
}

export function heartbeatInterval(ttl: number): number {
  return Math.max(250, Math.floor(ttl / 3));
}
