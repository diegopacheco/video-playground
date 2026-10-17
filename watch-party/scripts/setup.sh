#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
require bun
require ffmpeg

( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

MEDIA="$ROOT/frontend/public/media"
CLIP="$MEDIA/clip.mp4"
mkdir -p "$MEDIA"
if [ -s "$CLIP" ]; then
  log "clip already generated at frontend/public/media/clip.mp4"
else
  log "generating 120s clip with burned-in timecode"
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "testsrc2=size=1280x720:rate=30:duration=120" \
    -filter_complex "[0:v]split[bg][src];[src]crop=104:36:0:0,scale=728:252:flags=neighbor[tc];[bg][tc]overlay=(W-w)/2:(H-h)/2,format=yuv420p[v]" \
    -map "[v]" -c:v libx264 -preset veryfast -crf 26 -g 30 -keyint_min 30 -sc_threshold 0 \
    -movflags +faststart "$CLIP" || fail "ffmpeg clip generation failed"
fi

if ! grep -q '^\.run/$' "$ROOT/.gitignore"; then
  printf ".run/\n" >>"$ROOT/.gitignore"
fi

log "setup done"
