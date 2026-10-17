#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
( cd "$ROOT/backend" && cargo build --release ) || fail "backend build failed"

require bun
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

if ! grep -q '^\.run/$' "$ROOT/.gitignore" 2>/dev/null; then
  printf ".run/\n" >>"$ROOT/.gitignore"
fi

log "setup done"
