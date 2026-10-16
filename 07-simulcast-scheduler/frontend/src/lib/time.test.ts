import { describe, expect, test } from "bun:test";
import { cellText, formatCountdown, formatOffset, nextUnlockMs } from "./time";
import type { Board, Entitlement } from "../types";

function region(over: Partial<Entitlement>): Entitlement {
  return {
    episode_id: "e",
    country: "US",
    status: "locked",
    unlock_at: null,
    available_until: null,
    countdown_ms: null,
    reason: null,
    local_release: null,
    server_now: "",
    ...over,
  };
}

describe("formatCountdown", () => {
  test("rounds up so a viewer never sees 00:00:00 while still locked", () => {
    expect(formatCountdown(1)).toBe("00:00:01");
    expect(formatCountdown(999)).toBe("00:00:01");
  });

  test("shows days for long regional delays", () => {
    expect(formatCountdown(86_400_000 + 3_661_000)).toBe("1d 01:01:01");
  });

  test("never goes negative after unlock", () => {
    expect(formatCountdown(-5000)).toBe("00:00:00");
  });
});

describe("formatOffset", () => {
  test("signals simulated time travel direction", () => {
    expect(formatOffset(0)).toBe("real time");
    expect(formatOffset(-60_000)).toBe("-00:01:00");
    expect(formatOffset(3_600_000)).toBe("+01:00:00");
  });
});

describe("nextUnlockMs", () => {
  test("picks the soonest pending unlock across episodes so the jump lands on the next herd", () => {
    const board: Board = {
      server_now: "",
      offset_ms: 0,
      episodes: [
        { id: "a", title: "A", release_utc: "", regions: [region({ countdown_ms: 9000 }), region({ status: "available" })] },
        { id: "b", title: "B", release_utc: "", regions: [region({ countdown_ms: 4000 })] },
      ],
    };
    expect(nextUnlockMs(board)).toBe(4000);
  });

  test("returns null when nothing is pending", () => {
    expect(nextUnlockMs({ server_now: "", offset_ms: 0, episodes: [] })).toBeNull();
  });
});

describe("cellText", () => {
  test("distinguishes an expired license from a region that was never licensed", () => {
    expect(cellText(region({ status: "geo_blocked", reason: "license_expired" }))).toBe("EXPIRED");
    expect(cellText(region({ status: "geo_blocked", reason: "not_licensed" }))).toBe("BLOCKED");
  });
});
