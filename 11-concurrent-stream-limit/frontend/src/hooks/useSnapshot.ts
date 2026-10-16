import { useEffect, useState } from "react";
import { fetchSnapshot } from "../api/client";
import type { Snapshot } from "../types";

export interface TimedSnapshot {
  snapshot: Snapshot;
  fetchedAt: number;
}

export function useSnapshot(account: string, refreshKey: number): TimedSnapshot | null {
  const [state, setState] = useState<TimedSnapshot | null>(null);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      try {
        const snapshot = await fetchSnapshot(account);
        if (alive) {
          setState({ snapshot, fetchedAt: Date.now() });
        }
      } catch {
        if (alive) {
          setState(null);
        }
      }
    };
    void load();
    const timer = setInterval(() => void load(), 500);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [account, refreshKey]);

  return state;
}
