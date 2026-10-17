#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

[ -x "$ROOT/backend/target/release/server" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -d "$ROOT/frontend/node_modules" ] || fail "frontend dependencies missing, run ./scripts/setup.sh first"

start_bg backend "$ROOT/backend" env PORT="$backend_port" ROOM_RATE="${ROOM_RATE:-200}" ./target/release/server
wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" BACKEND_URL="$(service_url backend)" bun run dev
wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api/health"
printf "%-14s %s\n" waiting-room "$(service_url backend)/room/stats"
printf "%-14s %s\n" frontend "$(service_url frontend)"
