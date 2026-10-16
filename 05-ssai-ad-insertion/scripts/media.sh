#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

MEDIA="$ROOT/media"

encode() {
  local out video audio
  out="$1"
  video="$2"
  audio="$3"
  mkdir -p "$out"
  ffmpeg -y -loglevel error \
    -f lavfi -i "$video" -f lavfi -i "$audio" \
    -c:v libx264 -profile:v main -level 3.1 -pix_fmt yuv420p -r 24 -b:v 800k \
    -g 48 -keyint_min 48 -sc_threshold 0 \
    -c:a aac -ar 48000 -ac 2 -b:a 96k -shortest \
    -f hls -hls_time 2 -hls_playlist_type vod -hls_list_size 0 \
    -hls_segment_filename "$out/seg_%03d.ts" "$out/index.m3u8"
}

ad() {
  local id color seconds tone
  id="$1"
  color="$2"
  seconds="$3"
  tone="$4"
  encode "$MEDIA/ads/$id" \
    "color=c=$color:size=640x360:rate=24:duration=$seconds,drawbox=x='mod(t*160,560)':y=140:w=80:h=80:color=white@0.9:t=fill" \
    "sine=frequency=$tone:sample_rate=48000:duration=$seconds"
  log "ad $id ${seconds}s"
}

mark_cues() {
  awk '
    /^#EXTINF/ {
      if (n == 5) print "#EXT-X-CUE-OUT:DURATION=20"
      if (n == 15) print "#EXT-X-CUE-IN"
      if (n == 20) print "#EXT-X-DATERANGE:ID=\"ad-break-2\",CLASS=\"com.apple.hls.scte35.out\",START-DATE=\"2026-01-01T00:00:40.000Z\",PLANNED-DURATION=12,SCTE35-OUT=0xFC302000000000000000FFF00506FE000A8C000000000000"
      n++
    }
    { print }
  ' "$1" >"$2"
}

if [ -f "$MEDIA/content/content.m3u8" ] && [ -f "$MEDIA/ads/slate/index.m3u8" ]; then
  log "media already generated"
  exit 0
fi

require ffmpeg
rm -rf "$MEDIA"
encode "$MEDIA/content" "testsrc2=size=640x360:rate=24:duration=60" "sine=frequency=330:sample_rate=48000:duration=60"
mark_cues "$MEDIA/content/index.m3u8" "$MEDIA/content/content.m3u8"
log "content 60s with breaks at 10s (cue-out 20s) and 40s (daterange 12s)"
ad acme orange 10 660
ad globex teal 6 520
ad initech purple 8 440
ad umbrella crimson 4 880
ad slate gray 2 110
