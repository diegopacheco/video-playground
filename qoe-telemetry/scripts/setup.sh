#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require podman
require podman-compose
require cargo
require bun
require ffmpeg
require cmake

podman image exists docker.io/apache/kafka:4.3.1 || podman pull docker.io/apache/kafka:4.3.1

( cd "$ROOT/backend" && cargo build --bins ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

CLIP="$ROOT/frontend/public/media/clip"
if [ ! -f "$CLIP/master.m3u8" ]; then
  log "generating hls ladder into frontend/public/media/clip"
  mkdir -p "$CLIP"
  ( cd "$CLIP" && ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "testsrc2=size=1280x720:rate=30" \
    -f lavfi -i "sine=frequency=440:sample_rate=48000" \
    -t 30 \
    -filter_complex "[0:v]split=3[a][b][c];[a]scale=640:360[v0];[b]scale=960:540[v1];[c]scale=1280:720[v2]" \
    -map "[v0]" -map "[v1]" -map "[v2]" -map 1:a -map 1:a -map 1:a \
    -c:v libx264 -preset veryfast -g 60 -keyint_min 60 -sc_threshold 0 \
    -b:v:0 800k -b:v:1 1800k -b:v:2 3500k \
    -c:a aac -b:a 96k \
    -var_stream_map "v:0,a:0 v:1,a:1 v:2,a:2" \
    -master_pl_name master.m3u8 \
    -f hls -hls_time 2 -hls_playlist_type vod \
    -hls_segment_filename "v%v/seg_%03d.ts" "v%v/index.m3u8" ) || fail "clip generation failed"
fi

log "setup done"
