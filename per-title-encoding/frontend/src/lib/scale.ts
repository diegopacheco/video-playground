export type Scale = (value: number) => number;

export function linearScale(domain: [number, number], range: [number, number]): Scale {
  const [d0, d1] = domain;
  const [r0, r1] = range;
  const span = d1 - d0 || 1;
  return (v) => r0 + ((v - d0) / span) * (r1 - r0);
}

export function logScale(domain: [number, number], range: [number, number]): Scale {
  const inner = linearScale([Math.log10(domain[0]), Math.log10(domain[1])], range);
  return (v) => inner(Math.log10(Math.max(v, 1e-9)));
}

export function logTicks(min: number, max: number): number[] {
  const ticks: number[] = [];
  for (let exp = Math.floor(Math.log10(min)); exp <= Math.ceil(Math.log10(max)); exp++) {
    for (const m of [1, 2, 5]) {
      const v = m * 10 ** exp;
      if (v >= min && v <= max) ticks.push(v);
    }
  }
  return ticks;
}

export function linearTicks(min: number, max: number, count: number): number[] {
  const raw = (max - min) / Math.max(count, 1);
  const magnitude = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 5, 10].map((m) => m * magnitude).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = Math.ceil(min / step) * step; v <= max + step * 1e-9; v += step) {
    ticks.push(Number(v.toFixed(10)));
  }
  return ticks;
}

export function formatKbps(kbps: number): string {
  if (kbps >= 1000) return `${(kbps / 1000).toFixed(kbps >= 10000 ? 0 : 1)} Mbps`;
  return `${Math.round(kbps)} kbps`;
}

export function formatPct(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return `${(rounded === 0 ? 0 : rounded).toFixed(1)}%`;
}

export function formatScore(metric: string, value: number): string {
  return metric === "ssim" ? value.toFixed(4) : value.toFixed(1);
}
