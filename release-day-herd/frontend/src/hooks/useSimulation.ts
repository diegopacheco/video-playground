import { useCallback, useEffect, useState } from "react";
import { api } from "../api/client";
import type { SimConfig, SimReport } from "../types/sim";

export function useSimulation() {
  const [config, setConfig] = useState<SimConfig | null>(null);
  const [reports, setReports] = useState<SimReport[]>([]);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [elapsedMs, setElapsedMs] = useState<number | null>(null);

  useEffect(() => {
    api.defaultConfig().then(setConfig, (e: Error) => setError(e.message));
  }, []);

  const run = useCallback(async () => {
    if (!config) return;
    setRunning(true);
    setError(null);
    const started = performance.now();
    try {
      setReports(await api.compare(config));
      setElapsedMs(Math.round(performance.now() - started));
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRunning(false);
    }
  }, [config]);

  return { config, setConfig, reports, running, error, elapsedMs, run };
}
