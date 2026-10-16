import { describe, expect, test } from "bun:test";
import { latency, scoreParts } from "./score";
import type { Breakdown } from "../types/search";

const breakdown: Breakdown = {
  field: "english",
  similarity: 0.94,
  coverage: 1,
  edit_distance: 1,
  overlap: 0.9,
  boost: 0.95,
  popularity: 0.99,
  similarity_score: 0.67,
  coverage_score: 0.12,
  text_score: 0.79,
  popularity_score: 0.12,
  total: 0.91,
};

describe("scoreParts", () => {
  test("bar segments fill the whole bar so the breakdown never misleads", () => {
    const total = scoreParts(breakdown).reduce((acc, p) => acc + p.share, 0);
    expect(total).toBeCloseTo(100, 6);
  });

  test("segment values equal the server contributions so the bar sums to the score", () => {
    const sum = scoreParts(breakdown).reduce((acc, p) => acc + p.value, 0);
    expect(sum).toBeCloseTo(breakdown.total, 6);
  });

  test("a zero score does not divide by zero", () => {
    const zero = { ...breakdown, similarity_score: 0, coverage_score: 0, popularity_score: 0 };
    expect(scoreParts(zero).every((p) => p.share === 0)).toBe(true);
  });
});

describe("latency", () => {
  test("sub millisecond searches are shown in microseconds", () => {
    expect(latency(87)).toBe("87 µs");
    expect(latency(2500)).toBe("2.50 ms");
  });
});
