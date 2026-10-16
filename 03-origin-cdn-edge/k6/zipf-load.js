import http from "k6/http";
import { check } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";

const EDGE = __ENV.EDGE_URL || "http://127.0.0.1:6103";
const ORIGIN = __ENV.ORIGIN_URL || "http://127.0.0.1:8103";
const KEYS = Number(__ENV.KEYS || 500);
const SKEW = Number(__ENV.SKEW || 1.1);

const cdf = (() => {
  const weights = [];
  let total = 0;
  for (let rank = 1; rank <= KEYS; rank++) {
    total += 1 / Math.pow(rank, SKEW);
    weights.push(total);
  }
  return weights.map((w) => w / total);
})();

function zipfKey() {
  const u = Math.random();
  let lo = 0;
  let hi = cdf.length - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (cdf[mid] < u) lo = mid + 1;
    else hi = mid;
  }
  return `seg-${String(lo + 1).padStart(5, "0")}.ts`;
}

const cacheHit = new Rate("edge_cache_hit");
const statusCount = new Counter("edge_status");
const latencyHit = new Trend("latency_hit", true);
const latencyMiss = new Trend("latency_miss", true);
const latencyStale = new Trend("latency_stale", true);

export const options = {
  scenarios: {
    zipf: {
      executor: "constant-vus",
      vus: Number(__ENV.VUS || 50),
      duration: __ENV.DURATION || "40s",
    },
  },
  thresholds: {
    http_req_failed: ["rate<0.01"],
    edge_cache_hit: ["rate>0.5"],
  },
  summaryTrendStats: ["avg", "min", "med", "p(90)", "p(95)", "p(99)", "max"],
};

export function setup() {
  http.post(`${EDGE}/metrics/reset`);
  http.post(`${ORIGIN}/stats/reset`);
}

export default function () {
  const response = http.get(`${EDGE}/seg/${zipfKey()}`);
  const status = response.headers["X-Cache"] || "NONE";
  check(response, { "status 200": (r) => r.status === 200 });
  statusCount.add(1, { cache: status });
  cacheHit.add(status === "HIT" || status === "STALE");
  if (status === "HIT") latencyHit.add(response.timings.duration);
  else if (status === "STALE") latencyStale.add(response.timings.duration);
  else latencyMiss.add(response.timings.duration);
}

export function teardown() {
  const edge = http.get(`${EDGE}/metrics`).json();
  const origin = http.get(`${ORIGIN}/stats`).json();
  console.log(`EDGE_METRICS ${JSON.stringify(edge)}`);
  console.log(`ORIGIN_STATS ${JSON.stringify(origin)}`);
}

export function handleSummary(data) {
  const m = data.metrics;
  const trend = (name) => {
    const v = m[name] ? m[name].values : null;
    return v ? `avg=${v.avg.toFixed(2)}ms p50=${v.med.toFixed(2)}ms p95=${v["p(95)"].toFixed(2)}ms p99=${v["p(99)"].toFixed(2)}ms max=${v.max.toFixed(2)}ms` : "n/a";
  };
  const lines = [
    `requests=${m.http_reqs.values.count} rps=${m.http_reqs.values.rate.toFixed(1)} failed=${(m.http_req_failed.values.rate * 100).toFixed(2)}%`,
    `client_hit_ratio=${(m.edge_cache_hit.values.rate * 100).toFixed(2)}%`,
    `all     ${trend("http_req_duration")}`,
    `hit     ${trend("latency_hit")}`,
    `stale   ${trend("latency_stale")}`,
    `miss    ${trend("latency_miss")}`,
  ];
  return {
    stdout: `${lines.join("\n")}\n`,
    "k6/results/summary.json": JSON.stringify(data, null, 2),
    "k6/results/summary.txt": `${lines.join("\n")}\n`,
  };
}
