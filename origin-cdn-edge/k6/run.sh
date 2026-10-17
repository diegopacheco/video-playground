#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../scripts" && pwd)/common.sh"

require k6
port_up "$(service_port edge)" || fail "edge is not running, run ./scripts/start-all.sh first"
mkdir -p "$ROOT/k6/results"

k6 run \
  -e EDGE_URL="$(service_url edge)" \
  -e ORIGIN_URL="$(service_url origin)" \
  "$@" \
  "$ROOT/k6/zipf-load.js" 2>&1 | tee "$ROOT/k6/results/k6.log"

grep -o 'EDGE_METRICS.*}' "$ROOT/k6/results/k6.log" | sed 's/^EDGE_METRICS //; s/\\"/"/g; s/" source=console$//' >"$ROOT/k6/results/edge-metrics.json" || true
grep -o 'ORIGIN_STATS.*}' "$ROOT/k6/results/k6.log" | sed 's/^ORIGIN_STATS //; s/\\"/"/g' >"$ROOT/k6/results/origin-stats.json" || true
log "results in k6/results"
