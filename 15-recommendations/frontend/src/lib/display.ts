import type { Row, RowKind } from "../types";

const PALETTE: [string, string][] = [
  ["#ffd6e0", "#ffb3c6"],
  ["#d6e4ff", "#adc8ff"],
  ["#d9f2e6", "#a8e0c4"],
  ["#fff1c9", "#ffe08a"],
  ["#e8dcff", "#cdb6ff"],
  ["#ffe2cc", "#ffc499"],
  ["#d4f1f4", "#a3e1e8"],
  ["#f2e3d5", "#e2c4a6"],
];

export function posterColors(id: number): [string, string] {
  return PALETTE[Math.abs(id * 7 + 3) % PALETTE.length];
}

export function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0].toUpperCase())
    .join("");
}

const ORDER: RowKind[] = ["because", "top", "similar", "popular"];

export function visibleRows(rows: Row[]): Row[] {
  return rows
    .filter((r) => r.items.length > 0)
    .slice()
    .sort((a, b) => ORDER.indexOf(a.kind) - ORDER.indexOf(b.kind));
}

export function percent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

export function rowHint(kind: RowKind): string {
  switch (kind) {
    case "because":
      return "item-item collaborative filtering on co-watches";
    case "top":
      return "selected recommender";
    case "similar":
      return "content embedding nearest neighbors in pgvector";
    case "popular":
      return "most watched overall";
  }
}
