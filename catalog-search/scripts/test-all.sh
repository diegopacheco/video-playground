#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"
( cd "$ROOT/backend" && cargo run --release --quiet --bin bench ) || fail "latency bench failed"

require bun
( cd "$ROOT/frontend" && bun test src ) || fail "frontend tests failed"

log "tests passed"
