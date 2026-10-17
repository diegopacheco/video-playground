#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

REDIS_PORT="$(service_port redis)"
BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"

mkdir -p "$RUN/redis"
if port_up "$REDIS_PORT"; then
  log "redis already running"
else
  start_bg redis "$RUN/redis" redis-server --port "$REDIS_PORT" --bind 127.0.0.1 --save "" --appendonly no
fi
wait_port_up "$REDIS_PORT" 30 || fail "redis did not open port $REDIS_PORT, see $LOGS/redis.log"
log "redis up on $(service_url redis)"

( cd "$ROOT/backend" && cargo build -q ) || fail "backend build failed"
start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" REDIS_URL="redis://127.0.0.1:$REDIS_PORT" "$ROOT/backend/target/debug/streamlimit"
wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$FRONTEND_PORT" BACKEND_URL="http://127.0.0.1:$BACKEND_PORT" bun run dev
wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" redis "$(service_url redis)"
printf "%-14s %s\n" backend "$(service_url backend)/api"
printf "%-14s %s\n" frontend "$(service_url frontend)"
