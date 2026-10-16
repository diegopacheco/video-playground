#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require ffmpeg
require ffprobe
require cargo
require bun

( cd "$ROOT/backend" && cargo build --release ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

mkdir -p "$MEDIA"

if [ ! -f "$MEDIA/source.mp4" ]; then
  log "generating 30s 1080p24 source with ffmpeg"
  ffmpeg -y -v error -f lavfi \
    -i "testsrc2=size=1920x1080:rate=24:duration=30,noise=alls=12:allf=t" \
    -c:v libx264 -preset veryfast -crf 16 -pix_fmt yuv420p "$MEDIA/source.mp4" \
    || fail "source generation failed"
else
  log "source already present"
fi

if [ ! -f "$MEDIA/out/manifest.json" ]; then
  log "packaging ladder into CMAF, HLS and DASH"
  "$BINARY" package "$MEDIA/source.mp4" "$MEDIA/work" "$MEDIA/out" || fail "packaging failed"
else
  log "packaged output already present"
fi

log "setup done"
