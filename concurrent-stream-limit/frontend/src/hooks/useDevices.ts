import { useCallback, useEffect, useRef, useState } from "react";
import { acquire, heartbeat, release } from "../api/client";
import { heartbeatInterval } from "../lib/time";
import type { DeviceState, DeviceStatus, LogEntry } from "../types";

type Log = (text: string, tone: LogEntry["tone"]) => void;

export function deviceKey(account: string, device: string): string {
  return `${account}/${device}`;
}

export function useDevices(ttl: number, log: Log) {
  const [devices, setDevices] = useState<Record<string, DeviceState>>({});
  const timers = useRef(new Map<string, ReturnType<typeof setInterval>>());

  const set = useCallback((key: string, status: DeviceStatus, beat = false) => {
    setDevices((prev) => {
      const current = prev[key] ?? { status: "idle", beats: 0 };
      return { ...prev, [key]: { status, beats: beat ? current.beats + 1 : status === "playing" ? current.beats : 0 } };
    });
  }, []);

  const clear = useCallback((key: string) => {
    const timer = timers.current.get(key);
    if (timer !== undefined) {
      clearInterval(timer);
      timers.current.delete(key);
    }
  }, []);

  const beat = useCallback(
    async (account: string, device: string) => {
      const key = deviceKey(account, device);
      try {
        const lease = await heartbeat(account, device);
        if (!timers.current.has(key)) {
          return;
        }
        if (lease) {
          set(key, "playing", true);
        } else {
          clear(key);
          set(key, "kicked");
          log(`${device} lost its lease on heartbeat and stopped playback`, "bad");
        }
      } catch (e) {
        log(`${device} heartbeat failed: ${(e as Error).message}`, "warn");
      }
    },
    [clear, set, log],
  );

  const start = useCallback(
    async (account: string, device: string) => {
      const key = deviceKey(account, device);
      clear(key);
      try {
        const result = await acquire(account, device);
        if (result.status === "rejected") {
          set(key, "rejected");
          log(`${device} rejected: ${result.active}/${result.limit} streams already active`, "warn");
          return;
        }
        set(key, "playing");
        timers.current.set(key, setInterval(() => void beat(account, device), heartbeatInterval(ttl)));
        log(`${device} acquired a lease on ${account}`, "ok");
        for (const victim of result.evicted) {
          log(`${victim} was kicked to make room for ${device}`, "bad");
        }
      } catch (e) {
        log(`${device} acquire failed: ${(e as Error).message}`, "bad");
      }
    },
    [beat, clear, set, log, ttl],
  );

  const stop = useCallback(
    async (account: string, device: string) => {
      const key = deviceKey(account, device);
      clear(key);
      set(key, "idle");
      try {
        const released = await release(account, device);
        log(`${device} stopped and ${released ? "released its lease" : "had no lease to release"}`, "info");
      } catch (e) {
        log(`${device} release failed: ${(e as Error).message}`, "bad");
      }
    },
    [clear, set, log],
  );

  const crash = useCallback(
    (account: string, device: string) => {
      const key = deviceKey(account, device);
      clear(key);
      set(key, "crashed");
      log(`${device} crashed without releasing; its lease will expire by ttl`, "warn");
    },
    [clear, set, log],
  );

  useEffect(() => {
    const all = timers.current;
    return () => {
      for (const timer of all.values()) {
        clearInterval(timer);
      }
      all.clear();
    };
  }, []);

  return { devices, start, stop, crash };
}
