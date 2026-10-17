#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require podman
require podman-compose
require cargo
require bun

podman image exists docker.io/pgvector/pgvector:pg17 || podman pull docker.io/pgvector/pgvector:pg17

( cd "$ROOT/backend" && cargo build --release ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

log "setup done"
