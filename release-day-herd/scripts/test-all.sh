#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
require bun
( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"
( cd "$ROOT/frontend" && bun test src ) || fail "frontend tests failed"

backend_port="$(service_port backend)"
if ! command -v k6 >/dev/null 2>&1; then
  log "SKIPPED k6 live run: k6 is not installed"
elif ! port_up "$backend_port"; then
  log "SKIPPED k6 live run: backend is not running on $backend_port, run ./scripts/start-all.sh first"
else
  k6 run -e BASE_URL="$(service_url backend)" -e VIEWERS="${VIEWERS:-1000}" "$ROOT/k6/waiting-room.js" || fail "k6 waiting room run failed"
fi

log "tests passed"
