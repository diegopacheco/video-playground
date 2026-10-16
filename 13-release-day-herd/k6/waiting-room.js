import http from "k6/http";
import { check, sleep } from "k6";
import { Counter, Trend } from "k6/metrics";

const BASE = __ENV.BASE_URL || "http://localhost:8113";
const VIEWERS = Number(__ENV.VIEWERS || 1000);

const waitMs = Trend("room_wait_ms", true);
const played = Counter("room_played");
const polls = Counter("room_polls");

export const options = {
  scenarios: {
    midnight: {
      executor: "per-vu-iterations",
      vus: VIEWERS,
      iterations: 1,
      maxDuration: "2m",
    },
  },
  thresholds: {
    checks: ["rate==1"],
    room_played: [`count==${VIEWERS}`],
  },
};

export function setup() {
  const res = http.post(`${BASE}/room/reset`);
  check(res, { "room reset": (r) => r.status === 200 });
  return { rate: res.json("rate_per_s") };
}

export default function () {
  const started = Date.now();
  const joined = http.post(`${BASE}/room/join`);
  check(joined, { "joined with ticket": (r) => r.status === 200 && r.json("ticket") !== undefined });
  const ticket = joined.json("ticket");
  let token = null;
  while (token === null) {
    const status = http.get(`${BASE}/room/status/${ticket}`);
    polls.add(1);
    if (status.json("admitted")) {
      token = status.json("token");
    } else {
      sleep(Math.min(1, Math.max(0.2, status.json("eta_ms") / 1000)));
    }
  }
  const play = http.get(`${BASE}/room/play?token=${token}`);
  check(play, { "play allowed after admission": (r) => r.status === 200 });
  played.add(1);
  waitMs.add(Date.now() - started);
}

export function teardown(data) {
  const stats = http.get(`${BASE}/room/stats`).json();
  console.log(`room stats ${JSON.stringify(stats)}`);
  check(stats, {
    "every viewer admitted": (s) => s.admitted === VIEWERS,
    "admission never exceeded the rate": (s) => s.max_admitted_in_any_second <= data.rate,
  });
  const forged = http.get(`${BASE}/room/play?token=0000000000000001`);
  check(forged, { "forged token refused": (r) => r.status === 403 });
}
