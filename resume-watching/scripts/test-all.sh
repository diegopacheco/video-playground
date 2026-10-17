#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
require bun

for name in redis scylla; do
  port_up "$(service_port "$name")" || fail "$name is not running on $(service_port "$name"), the store tests need it, run ./scripts/start-all.sh first"
done

( cd "$ROOT/backend" && \
  REDIS_URL="redis://127.0.0.1:$(service_port redis)" \
  SCYLLA_NODE="127.0.0.1:$(service_port scylla)" \
  cargo test ) || fail "backend tests failed"

( cd "$ROOT/frontend" && bun test ) || fail "frontend tests failed"

log "tests passed"
