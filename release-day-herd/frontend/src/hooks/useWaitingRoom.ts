import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import type { RoomStats } from "../types/sim";

export interface Viewer {
  ticket: number;
  position: number;
  etaMs: number;
  joinedEtaMs: number;
  admitted: boolean;
  manifest: string | null;
}

export function useWaitingRoom() {
  const [viewers, setViewers] = useState<Viewer[]>([]);
  const [stats, setStats] = useState<RoomStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const viewersRef = useRef<Viewer[]>([]);
  viewersRef.current = viewers;

  const refresh = useCallback(async () => {
    try {
      const pending = viewersRef.current.filter((v) => v.manifest === null);
      const updated = await Promise.all(
        pending.map(async (v): Promise<Viewer> => {
          const s = await api.status(v.ticket);
          const manifest = s.token ? (await api.play(s.token)).manifest : null;
          return { ...v, position: s.position, etaMs: s.eta_ms, admitted: s.admitted, manifest };
        }),
      );
      const byTicket = new Map(updated.map((v) => [v.ticket, v]));
      setViewers((all) => all.map((v) => byTicket.get(v.ticket) ?? v));
      setStats(await api.stats());
    } catch (e) {
      setError((e as Error).message);
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 300);
    return () => clearInterval(id);
  }, [refresh]);

  const join = useCallback(async (count: number) => {
    try {
      const joined = await Promise.all(Array.from({ length: count }, () => api.join()));
      const fresh = joined.map((j) => ({
        ticket: j.ticket,
        position: j.position,
        etaMs: j.eta_ms,
        joinedEtaMs: j.eta_ms,
        admitted: false,
        manifest: null,
      }));
      setViewers((all) => [...fresh, ...all].sort((a, b) => b.ticket - a.ticket).slice(0, 400));
    } catch (e) {
      setError((e as Error).message);
    }
  }, []);

  const reset = useCallback(async () => {
    try {
      setStats(await api.reset());
      setViewers([]);
    } catch (e) {
      setError((e as Error).message);
    }
  }, []);

  return { viewers, stats, error, join, reset };
}
