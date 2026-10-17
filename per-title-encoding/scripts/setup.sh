#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require ffmpeg
require cargo
require bun

FILTERS="$(ffmpeg -hide_banner -filters 2>/dev/null || true)"
case "$FILTERS" in *libvmaf*) ;; *) fail "ffmpeg was built without libvmaf" ;; esac

( cd "$ROOT/backend" && cargo build --release ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

mkdir -p "$ROOT/data"
DATA_DIR="$ROOT/data" "$ROOT/backend/target/release/per-title" analyze-if-missing || fail "analysis failed"

log "setup done"
