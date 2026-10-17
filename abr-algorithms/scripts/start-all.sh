#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

[ -x "$ROOT/backend/target/debug/abr-server" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -d "$ROOT/frontend/node_modules" ] || fail "frontend dependencies missing, run ./scripts/setup.sh first"

if port_up "$backend_port"; then
  log "backend already listening on $backend_port"
else
  start_bg backend "$ROOT/backend" env PORT="$backend_port" "$ROOT/backend/target/debug/abr-server"
  wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
fi
log "backend up on $(service_url backend)/api"

if port_up "$frontend_port"; then
  log "frontend already listening on $frontend_port"
else
  start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" BACKEND_URL="$(service_url backend)" bun run dev
  wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
fi
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api"
printf "%-14s %s\n" frontend "$(service_url frontend)"
