#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

BIN="$ROOT/backend/target/release/per-title"
[ -x "$BIN" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -d "$ROOT/frontend/node_modules" ] || fail "frontend dependencies missing, run ./scripts/setup.sh first"

log "starting"

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"

if port_up "$BACKEND_PORT"; then
  log "backend already listening on $BACKEND_PORT"
else
  start_bg backend "$ROOT" env PORT="$BACKEND_PORT" DATA_DIR="$ROOT/data" "$BIN" serve
  wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
fi
log "backend up on $(service_url backend)/api"

if port_up "$FRONTEND_PORT"; then
  log "frontend already listening on $FRONTEND_PORT"
else
  start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$FRONTEND_PORT" BACKEND_PORT="$BACKEND_PORT" bun run dev
  wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
fi
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api/analysis"
printf "%-14s %s\n" frontend "$(service_url frontend)"
