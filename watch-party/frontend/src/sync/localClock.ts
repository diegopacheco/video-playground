const skew = Number(new URLSearchParams(window.location.search).get("skew") ?? "0");

export const clockSkewMs = Number.isFinite(skew) ? skew : 0;

export function localNow(): number {
  return performance.timeOrigin + performance.now() + clockSkewMs;
}
