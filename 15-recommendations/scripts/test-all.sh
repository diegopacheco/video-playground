#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

start_postgres

require cargo
export TEST_DATABASE_URL="postgres://$PG_USER:$PG_PASSWORD@localhost:$(service_port postgres)/recs_test"
( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"

require bun
( cd "$ROOT/frontend" && bun test ) || fail "frontend tests failed"

log "tests passed"
