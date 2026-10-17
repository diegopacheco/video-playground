#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"
BIN="$ROOT/backend/target/release/catalog-search"

log "starting"

[ -x "$BIN" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -d "$ROOT/frontend/node_modules" ] || fail "frontend dependencies missing, run ./scripts/setup.sh first"

if port_up "$BACKEND_PORT"; then
  log "backend already listening on $BACKEND_PORT"
else
  start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" "$BIN"
fi
wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

if port_up "$FRONTEND_PORT"; then
  log "frontend already listening on $FRONTEND_PORT"
else
  start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$FRONTEND_PORT" BACKEND_PORT="$BACKEND_PORT" bun run dev
fi
wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
for name in $(service_names); do
  printf "%-14s %s\n" "$name" "$(service_url "$name")"
done
