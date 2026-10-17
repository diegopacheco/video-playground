import { useEffect, useState } from "react";
import { fetchHistory, openWindowStream } from "../api/telemetry";
import { mergeWindows } from "../lib/metrics";
import type { WindowResult } from "../types/telemetry";

const KEEP_WINDOWS = 30;

export interface WindowsState {
  results: WindowResult[];
  connected: boolean;
  consumed: number;
  lateDropped: number;
  error: string | null;
}

export function useWindows(): WindowsState {
  const [results, setResults] = useState<WindowResult[]>([]);
  const [connected, setConnected] = useState(false);
  const [consumed, setConsumed] = useState(0);
  const [lateDropped, setLateDropped] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchHistory()
      .then((history) => setResults((cur) => mergeWindows(cur, history, KEEP_WINDOWS)))
      .catch((e: unknown) => setError(String(e)));
    return openWindowStream(
      (batch) => {
        setConsumed(batch.consumed);
        setLateDropped(batch.late_dropped);
        if (batch.results.length > 0) {
          setResults((cur) => mergeWindows(cur, batch.results, KEEP_WINDOWS));
        }
      },
      (ok) => {
        setConnected(ok);
        if (ok) setError(null);
      },
    );
  }, []);

  return { results, connected, consumed, lateDropped, error };
}
