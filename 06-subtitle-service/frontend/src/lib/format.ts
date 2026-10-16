import type { TagCounts } from "../types";

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

export function formatPercent(ratio: number): string {
  return `${Math.round(ratio * 100)}%`;
}

export function formatCounts(counts: TagCounts): string {
  const entries = Object.entries(counts);
  if (entries.length === 0) return "none";
  return entries.map(([tag, n]) => `\\${tag} x${n}`).join(", ");
}

export function extractCueStyles(vtt: string): string {
  const blocks = vtt.replace(/\r\n/g, "\n").split(/\n{2,}/);
  return blocks
    .filter((block) => block.startsWith("STYLE\n"))
    .map((block) => block.slice("STYLE\n".length))
    .join("\n");
}

export function fontsFor(locale: string): string[] {
  const latin = ["NotoSans-Regular.ttf", "NotoSans-Bold.ttf", "NotoSans-Italic.ttf", "NotoSerif-Bold.ttf"];
  const files = locale.startsWith("ja") ? ["NotoSansCJK-Regular.ttc"] : latin;
  return files.map((f) => `/media/fonts/${f}`);
}
