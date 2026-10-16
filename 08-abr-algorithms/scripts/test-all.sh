#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
require bun

( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"
( cd "$ROOT/backend" && cargo clippy --all-targets -- -D warnings ) || fail "backend clippy failed"
( cd "$ROOT/frontend" && bun run test ) || fail "frontend tests failed"
( cd "$ROOT/frontend" && bun run build ) || fail "frontend build failed"

backend_port="$(service_port backend)"
if port_up "$backend_port"; then
  code="$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'content-type: application/json' \
    -d '{"trace":"dropouts","algorithms":["throughput","bola","dynamic"]}' \
    "$(service_url backend)/api/simulate")"
  [ "$code" = "200" ] || fail "live simulate endpoint returned $code"
  log "live api check passed"
fi

log "tests passed"
