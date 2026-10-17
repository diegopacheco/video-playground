#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

[ -f "$MEDIA/hls/master.m3u8" ] || fail "media missing, run ./scripts/setup.sh first"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

( cd "$ROOT/backend" && cargo build -q ) || fail "backend build failed"
start_bg backend "$ROOT/backend" env SUBS_DIR="$ROOT/subs" MEDIA_DIR="$MEDIA" PORT="$backend_port" ./target/debug/subtitle-service
wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" BACKEND_URL="$(service_url backend)" bun run dev
wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" frontend "$(service_url frontend)"
printf "%-14s %s\n" backend "$(service_url backend)/api/tracks"
printf "%-14s %s\n" hls "$(service_url backend)/hls/master.m3u8"
