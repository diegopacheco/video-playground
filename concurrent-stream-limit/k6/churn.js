import http from "k6/http";
import { check } from "k6";
import { Counter, Trend } from "k6/metrics";

const BASE = __ENV.BASE_URL || "http://127.0.0.1:8111/api";
const ACCOUNT = "carol";
const LIMIT = 4;

const granted = new Counter("lease_granted");
const rejected = new Counter("lease_rejected");
const overLimit = new Counter("over_limit_observations");
const activeSeen = new Trend("active_leases_seen");

export const options = {
  scenarios: {
    churn: { executor: "constant-vus", vus: 64, duration: "15s", exec: "churn" },
    watch: { executor: "constant-vus", vus: 1, duration: "15s", exec: "watch" },
  },
  thresholds: {
    over_limit_observations: ["count==0"],
    http_req_failed: ["rate<0.01"],
  },
};

export function churn() {
  const device = `vu${__VU}`;
  const params = { responseCallback: http.expectedStatuses(200, 409, 410) };
  const res = http.post(`${BASE}/accounts/${ACCOUNT}/devices/${device}/acquire`, null, params);
  check(res, { "acquire answered 200 or 409": (r) => r.status === 200 || r.status === 409 });
  if (res.status === 200) {
    granted.add(1);
    http.post(`${BASE}/accounts/${ACCOUNT}/devices/${device}/heartbeat`, null, params);
    http.post(`${BASE}/accounts/${ACCOUNT}/devices/${device}/release`, null, params);
  } else {
    rejected.add(1);
  }
}

export function watch() {
  const res = http.get(`${BASE}/accounts/${ACCOUNT}/leases`);
  const n = res.json("leases").length;
  activeSeen.add(n);
  if (n > LIMIT) {
    overLimit.add(1);
  }
}
