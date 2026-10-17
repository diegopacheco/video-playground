#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"

require bun
( cd "$ROOT/frontend" && bun run test ) || fail "frontend tests failed"

log "tests passed"
