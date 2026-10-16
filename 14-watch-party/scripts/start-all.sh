#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

[ -s "$ROOT/frontend/public/media/clip.mp4" ] || fail "clip missing, run ./scripts/setup.sh first"

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"

if port_up "$BACKEND_PORT"; then
  log "backend already up"
else
  ( cd "$ROOT/backend" && cargo build -q ) || fail "backend build failed"
  start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" "$ROOT/backend/target/debug/watch-party"
  wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
fi
log "backend up on $(service_url backend)"

if port_up "$FRONTEND_PORT"; then
  log "frontend already up"
else
  start_bg frontend "$ROOT/frontend" env BACKEND_PORT="$BACKEND_PORT" FRONTEND_PORT="$FRONTEND_PORT" bun run dev
  wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
fi
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/health"
printf "%-14s %s\n" websocket "ws://localhost:$BACKEND_PORT/ws"
printf "%-14s %s\n" frontend "$(service_url frontend)"
