#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

MEDIA="$ROOT/media"
CLIP="$MEDIA/clip"
CLEARKEY_SYSTEM_ID="e2719d58-a985-b3c9-781a-b030af78d30e"

uuid() {
  printf "%s-%s-%s-%s-%s" "${1:0:8}" "${1:8:4}" "${1:12:4}" "${1:16:4}" "${1:20:12}"
}

inject_content_protection() {
  local kid_uuid tmp
  kid_uuid="$(uuid "$1")"
  tmp="$CLIP/stream.mpd.tmp"
  awk -v kid="$kid_uuid" -v sys="$CLEARKEY_SYSTEM_ID" '
    /<MPD / { sub(/<MPD /, "<MPD xmlns:cenc=\"urn:mpeg:cenc:2013\" ") }
    { print }
    /<AdaptationSet / {
      print "\t\t\t<ContentProtection schemeIdUri=\"urn:mpeg:dash:mp4protection:2011\" value=\"cenc\" cenc:default_KID=\"" kid "\"/>"
      print "\t\t\t<ContentProtection schemeIdUri=\"urn:uuid:" sys "\" value=\"ClearKey1.0\"/>"
    }
  ' "$CLIP/stream.mpd" >"$tmp"
  mv "$tmp" "$CLIP/stream.mpd"
}

generate_media() {
  local kid key
  kid="$(openssl rand -hex 16)"
  key="$(openssl rand -hex 16)"
  rm -rf "$MEDIA"
  mkdir -p "$CLIP"
  log "encoding source clip"
  ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i "testsrc2=size=1280x720:rate=30:duration=20" \
    -f lavfi -i "sine=frequency=440:duration=20" \
    -c:v libx264 -preset veryfast -profile:v main -g 60 -keyint_min 60 -sc_threshold 0 -pix_fmt yuv420p \
    -c:a aac -b:a 96k -shortest "$MEDIA/source.mp4"
  log "packaging CMAF DASH with cenc-aes-ctr"
  ( cd "$CLIP" && ffmpeg -hide_banner -loglevel error -y -i "$MEDIA/source.mp4" -c copy \
    -f dash -seg_duration 2 -use_template 1 -use_timeline 0 \
    -format_options "encryption_scheme=cenc-aes-ctr:encryption_key=$key:encryption_kid=$kid" \
    stream.mpd )
  rm -f "$MEDIA/source.mp4"
  inject_content_protection "$kid"
  cat >"$MEDIA/drm.json" <<EOF
{
  "session_secret": "$(openssl rand -hex 32)",
  "url_secret": "$(openssl rand -hex 32)",
  "contents": [ { "id": "clip", "kid": "$kid", "key": "$key" } ]
}
EOF
}

log "setup started"
require ffmpeg
require openssl
require cargo
require bun

if [ -f "$CLIP/stream.mpd" ] && [ -f "$MEDIA/drm.json" ] && grep -q ContentProtection "$CLIP/stream.mpd"; then
  log "encrypted media already present"
else
  generate_media
fi

( cd "$ROOT/backend" && cargo build --release -q ) || fail "backend build failed"
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

log "setup done"
