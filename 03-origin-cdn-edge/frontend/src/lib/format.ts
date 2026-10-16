export function formatPercent(ratio: number): string {
  if (!Number.isFinite(ratio)) return "0.0%";
  return `${(Math.min(Math.max(ratio, 0), 1) * 100).toFixed(1)}%`;
}

export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function formatCount(value: number): string {
  return value.toLocaleString("en-US");
}
