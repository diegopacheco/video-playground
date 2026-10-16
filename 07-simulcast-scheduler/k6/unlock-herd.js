import http from "k6/http";
import { check } from "k6";
import { Counter } from "k6/metrics";

const BASE = __ENV.BASE_URL || "http://localhost:8107";
const EPISODE = "neon-harbor-s1e1";
const UNLOCK_IN_SECS = Number(__ENV.UNLOCK_IN_SECS || 10);
const COUNTRIES = ["US", "BR", "GB", "DE", "IN", "JP", "KR", "CN", "AU"];
const JSON_HEADERS = { "content-type": "application/json" };

const available = new Counter("status_available");
const locked = new Counter("status_locked");
const blocked = new Counter("status_geo_blocked");
const throttled = new Counter("status_throttled");
const staleLocked = new Counter("stale_locked_after_unlock");
const playsGranted = new Counter("plays_granted");
const playsTooEarly = new Counter("plays_too_early");
const playsThrottled = new Counter("plays_throttled");

http.setResponseCallback(http.expectedStatuses(200, 425, 429, 451));

export const options = {
  scenarios: {
    herd: {
      executor: "constant-vus",
      vus: Number(__ENV.VUS || 300),
      duration: __ENV.DURATION || "25s",
    },
  },
  thresholds: {
    stale_locked_after_unlock: ["count==0"],
    http_req_failed: ["rate<0.01"],
    "http_req_duration{phase:before_unlock}": ["p(95)<200"],
    "http_req_duration{phase:unlock_window}": ["p(95)<200"],
    "http_req_duration{phase:after_unlock}": ["p(95)<200"],
  },
};

export function setup() {
  http.post(`${BASE}/api/clock`, JSON.stringify({ offset_ms: 0 }), { headers: JSON_HEADERS });
  const res = http.post(`${BASE}/api/episodes/${EPISODE}/release`, JSON.stringify({ in_secs: UNLOCK_IN_SECS }), { headers: JSON_HEADERS });
  http.post(`${BASE}/api/metrics/reset`);
  return { unlockAt: Date.parse(res.json("release_utc")) };
}

function phase(unlockAt) {
  const delta = Date.now() - unlockAt;
  if (delta < -1000) return "before_unlock";
  if (delta <= 1000) return "unlock_window";
  return "after_unlock";
}

export default function (data) {
  const country = COUNTRIES[Math.floor(Math.random() * COUNTRIES.length)];
  const tags = { phase: phase(data.unlockAt) };
  const params = { headers: { "x-country": country }, tags };
  const res = http.get(`${BASE}/api/episodes/${EPISODE}/entitlement`, params);

  if (res.status === 429) {
    throttled.add(1, tags);
    check(res, { "429 carries retry-after": (r) => r.headers["Retry-After"] !== undefined });
    return;
  }

  const body = res.json();
  if (body.status === "available") available.add(1, tags);
  if (body.status === "geo_blocked") blocked.add(1, tags);
  if (body.status === "locked") {
    locked.add(1, tags);
    if (Date.parse(body.server_now) >= Date.parse(body.unlock_at)) staleLocked.add(1);
    check(res, { "locked carries retry-after": (r) => r.headers["Retry-After"] !== undefined });
  }

  if (body.status !== "geo_blocked") {
    const play = http.post(`${BASE}/api/episodes/${EPISODE}/play`, null, params);
    if (play.status === 200) playsGranted.add(1, tags);
    if (play.status === 425) playsTooEarly.add(1, tags);
    if (play.status === 429) playsThrottled.add(1, tags);
  }
}

export function teardown() {
  console.log(`server metrics ${http.get(`${BASE}/api/metrics`).body}`);
}
