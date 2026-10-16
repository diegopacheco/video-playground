import { useEffect, useState } from "react";
import { fetchBeacons } from "../api/client";
import type { Beacon } from "../types";

export function useBeacons(session: string, intervalMs = 1000): Beacon[] {
  const [beacons, setBeacons] = useState<Beacon[]>([]);

  useEffect(() => {
    let active = true;
    setBeacons([]);
    const load = () =>
      fetchBeacons(session)
        .then((b) => {
          if (active) setBeacons(b);
        })
        .catch(() => undefined);
    load();
    const timer = setInterval(load, intervalMs);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [session, intervalMs]);

  return beacons;
}
