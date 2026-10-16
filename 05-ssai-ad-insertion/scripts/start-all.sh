#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

[ -f "$ROOT/media/content/content.m3u8" ] || fail "media missing, run ./scripts/setup.sh first"
[ -x "$ROOT/backend/target/debug/ssai" ] || fail "backend not built, run ./scripts/setup.sh first"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

if port_up "$backend_port"; then
  log "backend already listening on $backend_port"
else
  start_bg backend "$ROOT/backend" env PORT="$backend_port" MEDIA_DIR="$ROOT/media" "$ROOT/backend/target/debug/ssai"
fi
wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

if port_up "$frontend_port"; then
  log "frontend already listening on $frontend_port"
else
  start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" BACKEND_PORT="$backend_port" bun run dev
fi
wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api/manifest.m3u8?user=alice"
printf "%-14s %s\n" frontend "$(service_url frontend)"
