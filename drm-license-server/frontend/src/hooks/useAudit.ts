import { useEffect, useState } from "react";
import { fetchAudit } from "../api/client";
import type { AuditEvent } from "../types";

export function useAudit(intervalMs: number): AuditEvent[] {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  useEffect(() => {
    let alive = true;
    const tick = () =>
      fetchAudit()
        .then((e) => alive && setEvents(e))
        .catch(() => undefined);
    tick();
    const timer = setInterval(tick, intervalMs);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [intervalMs]);
  return events;
}
