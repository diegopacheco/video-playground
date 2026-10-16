#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

start_postgres
log "postgres up on $(service_url postgres)"

( cd "$ROOT/backend" && cargo build --release -q ) || fail "backend build failed"
export DATABASE_URL="postgres://$PG_USER:$PG_PASSWORD@localhost:$(service_port postgres)/$PG_DB"
export PORT="$(service_port backend)"
start_bg backend "$ROOT/backend" "$ROOT/backend/target/release/recs"
wait_port_up "$(service_port backend)" 60 || fail "backend did not open port $(service_port backend), see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

export BACKEND_PORT="$(service_port backend)"
export FRONTEND_PORT="$(service_port frontend)"
start_bg frontend "$ROOT/frontend" bun run dev
wait_port_up "$(service_port frontend)" 60 || fail "frontend did not open port $(service_port frontend), see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" frontend "$(service_url frontend)"
printf "%-14s %s\n" backend "$(service_url backend)/api"
printf "%-14s %s\n" postgres "$(service_url postgres)/$PG_DB"
