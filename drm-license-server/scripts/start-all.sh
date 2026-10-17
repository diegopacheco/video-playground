#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

[ -f "$ROOT/media/drm.json" ] || fail "media missing, run ./scripts/setup.sh first"
[ -x "$ROOT/backend/target/release/drm-server" ] || fail "backend binary missing, run ./scripts/setup.sh first"

log "starting"

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"
export BACKEND_PORT FRONTEND_PORT

if port_up "$BACKEND_PORT"; then
  log "backend already up"
else
  start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" MEDIA_DIR="$ROOT/media" "$ROOT/backend/target/release/drm-server"
  wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
fi
log "backend up on $(service_url backend)"

if port_up "$FRONTEND_PORT"; then
  log "frontend already up"
else
  start_bg frontend "$ROOT/frontend" bun run dev
  wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
fi
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" frontend "$(service_url frontend)"
printf "%-14s %s\n" backend "$(service_url backend)/api"
printf "%-14s %s\n" license "$(service_url backend)/api/license/clearkey"
