#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "setup started"

require cargo
require bun
require ffmpeg
require podman

mkdir -p "$MEDIA/burned" "$MEDIA/fonts" "$MEDIA/hls" "$MEDIA/metrics"

if ! podman image exists "$IMAGE"; then
  log "building $IMAGE (ffmpeg with libass and Noto fonts)"
  podman build -t "$IMAGE" -f "$ROOT/containers/Containerfile" "$ROOT/containers"
fi

if [ ! -f "$MEDIA/source.mp4" ]; then
  log "generating source video"
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "gradients=s=1280x720:d=16:speed=0.015:rate=30" \
    -f lavfi -i "sine=frequency=220:duration=16" \
    -c:v libx264 -preset veryfast -crf 23 -g 60 -bf 0 -pix_fmt yuv420p \
    -c:a aac -b:a 96k -shortest -movflags +faststart "$MEDIA/source.mp4"
fi

for font in NotoSans-Regular.ttf NotoSans-Bold.ttf NotoSans-Italic.ttf NotoSerif-Regular.ttf NotoSerif-Bold.ttf NotoSansCJK-Regular.ttc; do
  if [ ! -f "$MEDIA/fonts/$font" ]; then
    podman run --rm -v "$MEDIA/fonts:/out" "$IMAGE" cp "/usr/share/fonts/noto/$font" /out/
  fi
done

encode() {
  local filter out logfile
  filter="$1"
  out="$2"
  logfile="$3"
  podman run --rm -v "$MEDIA:/media" -v "$ROOT/subs:/subs:ro" "$IMAGE" \
    ffmpeg -hide_banner -benchmark -y -i /media/source.mp4 -vf "$filter" \
    -c:v libx264 -preset veryfast -crf 23 -g 60 -pix_fmt yuv420p -c:a copy -movflags +faststart \
    "$out" >"$logfile" 2>&1 || fail "encode failed, see $logfile"
}

if [ ! -f "$MEDIA/metrics/plain.log" ]; then
  log "encoding baseline without subtitles"
  encode "null" /media/metrics/plain.mp4 "$MEDIA/metrics/plain.log"
fi

for locale in $LOCALES; do
  if [ ! -f "$MEDIA/burned/$locale.mp4" ]; then
    log "burning $locale subtitles with libass"
    encode "ass=/subs/$locale.ass:fontsdir=/media/fonts" "/media/burned/$locale.mp4" "$MEDIA/metrics/burn-$locale.log"
  fi
done

render_only() {
  podman run --rm -v "$MEDIA:/media" -v "$ROOT/subs:/subs:ro" "$IMAGE" \
    ffmpeg -hide_banner -benchmark -i /media/source.mp4 -vf "$1" -an -f null - >"$2" 2>&1 || fail "render failed, see $2"
}

if [ ! -f "$MEDIA/metrics/render-none.log" ]; then
  render_only "null" "$MEDIA/metrics/render-none.log"
  for locale in $LOCALES; do
    render_only "ass=/subs/$locale.ass:fontsdir=/media/fonts" "$MEDIA/metrics/render-$locale.log"
  done
fi

if [ ! -f "$MEDIA/hls/master.m3u8" ]; then
  log "packaging HLS video rendition"
  ffmpeg -hide_banner -loglevel error -y -i "$MEDIA/source.mp4" -c copy \
    -f hls -hls_time 4 -hls_playlist_type vod -hls_segment_type fmp4 \
    -hls_fmp4_init_filename init.mp4 -hls_segment_filename "$MEDIA/hls/seg_%03d.m4s" \
    -master_pl_name master.m3u8 "$MEDIA/hls/video.m3u8"
fi

( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

log "setup done"
