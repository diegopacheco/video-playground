#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
require bun

( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"
( cd "$ROOT/frontend" && bun run test ) || fail "frontend tests failed"

log "tests passed"
