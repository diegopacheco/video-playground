#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

BIN="$ROOT/backend/target/release"
if [ ! -x "$BIN/origin" ] || [ ! -x "$BIN/edge" ]; then
  ( cd "$ROOT/backend" && cargo build --release --bins ) || fail "backend build failed"
fi

ORIGIN_PORT="$(service_port origin)"
EDGE_PORT="$(service_port edge)"
FRONTEND_PORT="$(service_port frontend)"
ORIGIN_URL="$(service_url origin)"
EDGE_URL="$(service_url edge)"
export ORIGIN_PORT EDGE_PORT FRONTEND_PORT ORIGIN_URL EDGE_URL

start_bg origin "$ROOT/backend" "$BIN/origin"
wait_port_up "$ORIGIN_PORT" 60 || fail "origin did not open port $ORIGIN_PORT, see $LOGS/origin.log"
log "origin up on $ORIGIN_URL"

start_bg edge "$ROOT/backend" "$BIN/edge"
wait_port_up "$EDGE_PORT" 60 || fail "edge did not open port $EDGE_PORT, see $LOGS/edge.log"
log "edge up on $EDGE_URL"

start_bg frontend "$ROOT/frontend" bun run dev
wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" origin "$ORIGIN_URL/seg/{key}"
printf "%-14s %s\n" edge "$EDGE_URL/seg/{key}"
printf "%-14s %s\n" metrics "$EDGE_URL/metrics"
printf "%-14s %s\n" frontend "$(service_url frontend)"
