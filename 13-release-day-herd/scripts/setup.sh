#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
require bun
( cd "$ROOT/backend" && cargo fetch && cargo build --release --bin server --bin report ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

if ! command -v k6 >/dev/null 2>&1; then
  log "k6 not found, the live load test in test-all.sh will be skipped"
fi

log "setup done"
