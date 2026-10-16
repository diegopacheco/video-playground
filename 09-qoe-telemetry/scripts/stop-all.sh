#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "stopping"

stop_bg frontend
stop_bg loadgen
stop_bg backend

require podman-compose
KAFKA_PORT="$(service_port kafka)" podman-compose down >"$LOGS/compose.log" 2>&1 || fail "podman-compose down failed, see $LOGS/compose.log"
wait_port_down "$(service_port kafka)" 30 || true

for name in $(service_names); do
  port="$(service_port "$name")"
  if port_up "$port"; then
    fail "$name still listening on $port"
  fi
done

log "stopped"
