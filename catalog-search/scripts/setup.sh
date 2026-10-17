#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
( cd "$ROOT/backend" && cargo build --release --bins ) || fail "backend build failed"

require bun
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

log "setup done"
