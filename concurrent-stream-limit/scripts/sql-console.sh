#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

port="$(service_port redis)"
[ -n "$port" ] || fail "redis is not declared in scripts/ports.env"
port_up "$port" || fail "redis is not running on $port, run ./scripts/start-all.sh first"
require redis-cli

redis-cli -h 127.0.0.1 -p "$port" "$@"
