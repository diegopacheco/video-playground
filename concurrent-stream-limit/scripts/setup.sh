#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require redis-server
require redis-cli
require cargo
require bun

( cd "$ROOT/backend" && cargo fetch && cargo build ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

if [ ! -f "$ROOT/.gitignore" ] || ! grep -q '^\.run/$' "$ROOT/.gitignore"; then
  printf ".run/\n" >>"$ROOT/.gitignore"
fi

log "setup done"
