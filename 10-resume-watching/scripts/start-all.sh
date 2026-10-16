#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

STORE="${PLAYHEAD_STORE:-redis}"
FLUSH="${FLUSH_SECS:-15}"
SERVER="$ROOT/backend/target/release/server"

wait_scylla() {
  local tries
  tries=60
  while [ "$tries" -gt 0 ]; do
    if podman logs vp10-scylla 2>&1 | grep -q "Starting listening for CQL clients on"; then return 0; fi
    sleep 1
    tries=$((tries - 1))
  done
  return 1
}

log "starting with store $STORE"
require podman-compose
[ -x "$SERVER" ] || fail "backend is not built, run ./scripts/setup.sh first"

podman-compose -f "$ROOT/podman-compose.yml" up -d >"$LOGS/compose.log" 2>&1 || fail "podman-compose up failed, see $LOGS/compose.log"
wait_port_up "$(service_port redis)" 60 || fail "redis did not open port $(service_port redis)"
log "redis up on $(service_url redis)"
wait_scylla || fail "scylla did not start listening for CQL clients, see podman logs vp10-scylla"
log "scylla up on $(service_url scylla)"

start_bg backend "$ROOT/backend" env \
  PORT="$(service_port backend)" \
  PLAYHEAD_STORE="$STORE" \
  FLUSH_SECS="$FLUSH" \
  REDIS_URL="redis://127.0.0.1:$(service_port redis)" \
  SCYLLA_NODE="127.0.0.1:$(service_port scylla)" \
  "$SERVER"
wait_port_up "$(service_port backend)" 60 || fail "backend did not open port $(service_port backend), see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg frontend "$ROOT/frontend" env \
  FRONTEND_PORT="$(service_port frontend)" \
  BACKEND_URL="http://127.0.0.1:$(service_port backend)" \
  bun run dev
wait_port_up "$(service_port frontend)" 60 || fail "frontend did not open port $(service_port frontend), see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
for name in $(service_names); do
  printf "%-14s %s\n" "$name" "$(service_url "$name")"
done
