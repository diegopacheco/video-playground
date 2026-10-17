#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require podman
require podman-compose
require cargo
require bun

podman-compose -f "$ROOT/podman-compose.yml" pull
( cd "$ROOT/backend" && cargo build --release --bins ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

log "setup done"
