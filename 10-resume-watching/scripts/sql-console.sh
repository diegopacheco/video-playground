#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

TARGET="${1:-both}"
[ "$#" -gt 0 ] && shift

redis_console() {
  local port
  port="$(service_port redis)"
  port_up "$port" || fail "redis is not running on $port, run ./scripts/start-all.sh first"
  log "redis-cli on $(service_url redis), try: KEYS ph:*  HGETALL ph:profile-ana"
  if command -v redis-cli >/dev/null 2>&1; then
    redis-cli -h 127.0.0.1 -p "$port" "$@"
  else
    podman exec -it vp10-redis redis-cli "$@"
  fi
}

scylla_console() {
  local port
  port="$(service_port scylla)"
  port_up "$port" || fail "scylla is not running on $port, run ./scripts/start-all.sh first"
  log "cqlsh on $(service_url scylla), try: SELECT * FROM resume.playheads WHERE profile_id = 'profile-ana';"
  local ip tty
  ip="$(podman exec vp10-scylla hostname -i | awk '{print $1}')"
  tty="-i"
  if [ -t 0 ]; then tty="-it"; fi
  podman exec "$tty" vp10-scylla cqlsh "$ip" "$port" "$@"
}

case "$TARGET" in
  redis) redis_console "$@" ;;
  scylla|cql) scylla_console "$@" ;;
  both)
    redis_console
    scylla_console
    ;;
  *) fail "usage: sql-console.sh [redis|scylla|both] [client args]" ;;
esac
