#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
require bun
( cd "$ROOT/backend" && cargo fetch && cargo build --release --bins ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"
mkdir -p "$ROOT/k6/results"

if ! command -v k6 >/dev/null 2>&1; then
  log "k6 not found, load test in k6/run.sh will not run"
fi

log "setup done"
