#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

REDIS_PORT="$(service_port redis)"
port_up "$REDIS_PORT" || fail "redis is not running on $REDIS_PORT, run ./scripts/start-all.sh first"

( cd "$ROOT/backend" && REDIS_URL="redis://127.0.0.1:$REDIS_PORT" cargo test ) || fail "backend tests failed"
( cd "$ROOT/frontend" && bun test src ) || fail "frontend tests failed"

log "tests passed"
