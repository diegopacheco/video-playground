#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

[ -x "$BINARY" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -f "$MEDIA/out/manifest.json" ] || fail "packaged media missing, run ./scripts/setup.sh first"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

start_bg backend "$ROOT/backend" "$BINARY" serve "$MEDIA/out" "$backend_port"
wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" VITE_API_URL="http://localhost:$backend_port" bun run dev
wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
for name in $(service_names); do
  printf "%-14s %s\n" "$name" "$(service_url "$name")"
done
printf "%-14s %s\n" "hls" "http://localhost:$backend_port/media/master.m3u8"
printf "%-14s %s\n" "dash" "http://localhost:$backend_port/media/manifest.mpd"
