import { describe, expect, test } from "bun:test";
import { initials, percent, posterColors, visibleRows } from "./display";
import type { Row, Title } from "../types";

const title: Title = { id: 1, name: "Neon Ronin", genres: ["Action"], tags: [], synopsis: "", year: 2020, is_new: false };

function row(kind: Row["kind"], count: number): Row {
  return { kind, label: kind, anchor: null, items: Array.from({ length: count }, () => ({ title, score: 1 })) };
}

describe("visibleRows", () => {
  test("hides empty rows so a cold-start user never sees a blank shelf", () => {
    const rows = visibleRows([row("top", 3), row("because", 0), row("popular", 2)]);
    expect(rows.map((r) => r.kind)).toEqual(["top", "popular"]);
  });

  test("puts the personal co-watch row first because it explains itself best", () => {
    const rows = visibleRows([row("popular", 1), row("similar", 1), row("top", 1), row("because", 1)]);
    expect(rows.map((r) => r.kind)).toEqual(["because", "top", "similar", "popular"]);
  });
});

describe("display helpers", () => {
  test("poster colors are stable per title so a card looks the same in every row", () => {
    expect(posterColors(42)).toEqual(posterColors(42));
  });

  test("initials use the first two words", () => {
    expect(initials("Shattered Lunar Archive")).toBe("SL");
  });

  test("percent keeps one decimal", () => {
    expect(percent(0.30541)).toBe("30.5%");
  });
});
