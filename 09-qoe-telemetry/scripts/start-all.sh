#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

require podman-compose
KAFKA_PORT="$(service_port kafka)" podman-compose up -d >"$LOGS/compose.log" 2>&1 || fail "podman-compose up failed, see $LOGS/compose.log"
wait_port_up "$(service_port kafka)" 60 || fail "kafka did not open port $(service_port kafka)"
log "kafka up on $(service_url kafka)"

[ -x "$ROOT/backend/target/debug/qoe" ] || fail "backend not built, run ./scripts/setup.sh first"

start_bg backend "$ROOT/backend" env \
  BACKEND_PORT="$(service_port backend)" \
  KAFKA_BROKERS="localhost:$(service_port kafka)" \
  "$ROOT/backend/target/debug/qoe"
wait_port_up "$(service_port backend)" 60 || fail "backend did not open port $(service_port backend), see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg loadgen "$ROOT/backend" env \
  INGEST_URL="http://localhost:$(service_port backend)/api/beacons" \
  SESSIONS="${SESSIONS:-60}" \
  "$ROOT/backend/target/debug/loadgen"

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
