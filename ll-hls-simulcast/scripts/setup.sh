#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
require bun
require ffmpeg

ffmpeg -hide_banner -encoders 2>/dev/null | grep -q libx264 || fail "ffmpeg must be built with libx264"
ffmpeg -hide_banner -filters 2>/dev/null | grep -q " overlay " || fail "ffmpeg must have the overlay filter"

( cd "$ROOT/backend" && cargo build --release ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

if [ ! -f "$ROOT/.gitignore" ] || ! grep -q '^\.run/$' "$ROOT/.gitignore"; then
  printf ".run/\n" >>"$ROOT/.gitignore"
fi

log "setup done"
