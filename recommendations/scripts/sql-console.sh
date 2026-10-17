#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

port="$(service_port postgres)"
port_up "$port" || fail "postgres is not running on $port, run ./scripts/start-all.sh first"

if command -v psql >/dev/null 2>&1; then
  PGPASSWORD="$PG_PASSWORD" psql -h localhost -p "$port" -U "$PG_USER" -d "$PG_DB" "$@"
elif [ -t 0 ]; then
  podman exec -it "$PG_CONTAINER" psql -U "$PG_USER" -d "$PG_DB" "$@"
else
  podman exec -i "$PG_CONTAINER" psql -U "$PG_USER" -d "$PG_DB" "$@"
fi
