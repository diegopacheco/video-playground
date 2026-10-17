import { useCallback, useEffect, useRef, useState } from "react";
import * as dashjs from "dashjs";
import { ApiError, LICENSE_URL, createPlayback, createSession } from "../api/client";
import { maskKeys, tamperSignature } from "../scenarios";
import type { LicenseExchange, LogEntry, LogLevel, Scenario, Session } from "../types";

function decode(data: unknown): string {
  if (data instanceof ArrayBuffer) return new TextDecoder().decode(data);
  if (ArrayBuffer.isView(data)) return new TextDecoder().decode(data);
  return typeof data === "string" ? data : JSON.stringify(data);
}

function pretty(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return text;
  }
}

export function useDrmPlayer() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const playerRef = useRef<dashjs.MediaPlayerClass | null>(null);
  const nextId = useRef(0);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [session, setSession] = useState<Session | null>(null);
  const [manifestUrl, setManifestUrl] = useState<string>("");
  const [license, setLicense] = useState<LicenseExchange>({ request: "", response: "" });
  const [status, setStatus] = useState<LogLevel>("info");

  const log = useCallback((level: LogLevel, text: string) => {
    nextId.current += 1;
    const entry = { id: nextId.current, level, text };
    setLogs((prev) => [...prev, entry]);
    if (level !== "info") setStatus(level);
  }, []);

  const stop = useCallback(() => {
    playerRef.current?.destroy();
    playerRef.current = null;
  }, []);

  useEffect(() => stop, [stop]);

  const attach = useCallback(
    (url: string, token: string) => {
      const video = videoRef.current;
      if (!video) return;
      const player = dashjs.MediaPlayer().create();
      player.updateSettings({ debug: { logLevel: 0 }, streaming: { retryAttempts: { MPD: 0, MediaSegment: 0, InitializationSegment: 0 } } });
      player.initialize(video, undefined, true);
      player.setProtectionData({
        "org.w3.clearkey": { serverURL: LICENSE_URL, httpRequestHeaders: { Authorization: `Bearer ${token}` } },
      });
      player.registerLicenseRequestFilter((req) => {
        setLicense({ request: pretty(decode(req.data)), response: "" });
        log("info", `license request POST ${req.url}`);
        return Promise.resolve();
      });
      player.registerLicenseResponseFilter((res) => {
        setLicense((prev) => ({ ...prev, response: maskKeys(decode(res.data)) }));
        return Promise.resolve();
      });
      player.on(dashjs.MediaPlayer.events.KEY_SYSTEM_SELECTED, () => log("info", "EME key system selected: org.w3.clearkey"));
      player.on(dashjs.MediaPlayer.events.LICENSE_REQUEST_COMPLETE, (e: dashjs.LicenseRequestCompleteEvent) => {
        if (e.error) log("error", `license denied: ${e.error.message}`);
        else log("ok", "license granted, keys loaded into CDM session");
      });
      player.on(dashjs.MediaPlayer.events.PLAYBACK_PLAYING, () => log("ok", "decrypted frames playing"));
      player.on(dashjs.MediaPlayer.events.ERROR, (e: dashjs.MediaPlayerErrorEvent) => {
        log("error", `player error ${e.error.code}: ${e.error.message}`);
      });
      player.attachSource(url);
      playerRef.current = player;
    },
    [log],
  );

  const run = useCallback(
    async (scenario: Scenario) => {
      stop();
      setLogs([]);
      setLicense({ request: "", response: "" });
      setManifestUrl("");
      setStatus("info");
      log("info", `scenario: ${scenario.title}`);
      try {
        const s = await createSession("alice", scenario.entitled, scenario.sessionExpired);
        setSession(s);
        log("info", `session issued, ent=${JSON.stringify(s.claims.ent)}`);
        const playback = await createPlayback(s.token, "clip", scenario.urlExpired);
        const url = scenario.tamper ? tamperSignature(playback.manifestUrl) : playback.manifestUrl;
        setManifestUrl(url);
        log("info", `signed manifest url${scenario.tamper ? " (tampered)" : ""}`);
        attach(url, s.token);
      } catch (e) {
        const msg = e instanceof ApiError ? `${e.status} ${e.code}: ${e.message}` : String(e);
        log("error", `api refused: ${msg}`);
      }
    },
    [attach, log, stop],
  );

  return { videoRef, run, logs, session, manifestUrl, license, status };
}
