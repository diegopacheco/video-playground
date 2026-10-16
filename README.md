# video-playground

Hands-on POCs of how an anime streaming platform works: encoding, delivery, playback, subtitles, DRM and the services around them. Every project has a Rust 2024 backend, a React 19 + TypeScript 7 UI built with Vite and Bun, its own README, scripts and printscreens.

## 🎬 Encoding & Packaging

Turning one source file into many qualities a player can switch between.
Anime and live action compress very differently, so the ladder matters.

* [01-hls-dash-packager](01-hls-dash-packager/) - 240p to 1080p ladder with aligned keyframes, CMAF segments, HLS and DASH manifests, hls.js and dash.js players
* [02-per-title-encoding](02-per-title-encoding/) - VMAF and SSIM convex hull per title, custom ladder for anime-like vs live-action-like content

## 🌐 Delivery & Live

Getting segments to millions of screens fast, from origin to edge to a live simulcast.

* [03-origin-cdn-edge](03-origin-cdn-edge/) - Origin plus edge cache with LRU, request coalescing, stale-while-revalidate and a k6 hit ratio report
* [04-ll-hls-simulcast](04-ll-hls-simulcast/) - Low-latency HLS with partial segments, blocking playlist reload, preload hints and glass-to-glass latency
* [05-ssai-ad-insertion](05-ssai-ad-insertion/) - Server-side ad insertion on SCTE-35 cues with discontinuity tags and per-user ad decisions

## 💬 Subtitles & Releases

Anime lives on styled subtitles and on episodes that unlock at the same instant worldwide.

* [06-subtitle-service](06-subtitle-service/) - ASS/SSA to WebVTT per locale, compared with JASSUB styled rendering and burned-in subtitles
* [07-simulcast-scheduler](07-simulcast-scheduler/) - Per-region unlock times with time zones, geo-blocking and herd protection at release

## 📺 Playback & QoE

What happens inside the player and how we know if viewers are having a good time.

* [08-abr-algorithms](08-abr-algorithms/) - Throughput, BOLA and hybrid ABR replayed over network traces, rebuffer and bitrate compared
* [09-qoe-telemetry](09-qoe-telemetry/) - Player beacons into Kafka, Rust windowed aggregation, live QoE dashboard per CDN and title
* [10-resume-watching](10-resume-watching/) - Playhead heartbeats with write coalescing and last-write-wins on Redis vs ScyllaDB

## 🔐 Access & Protection

Who may watch, on how many screens, and how the content stays encrypted.

* [11-concurrent-stream-limit](11-concurrent-stream-limit/) - Max N streams per account with Redis Lua leases, heartbeats and TTL crash recovery
* [12-drm-license-server](12-drm-license-server/) - Clear Key license server, CENC encrypted CMAF, signed session tokens and expiring segment URLs

## 🚀 Scale & Social

Surviving a million people pressing play at 00:00, and watching together.

* [13-release-day-herd](13-release-day-herd/) - 1M viewer simulation comparing jittered retries, CDN pre-warming and a waiting room
* [14-watch-party](14-watch-party/) - WebSocket watch party with a leader clock, NTP-style sync and drift correction

## 🔎 Discovery

Helping viewers find the next show, in any language or script.

* [15-recommendations](15-recommendations/) - Item-item collaborative filtering, pgvector embeddings for similar anime and cold start
* [16-catalog-search](16-catalog-search/) - Own n-gram index for romaji, kana, kanji and English titles with typo tolerance
