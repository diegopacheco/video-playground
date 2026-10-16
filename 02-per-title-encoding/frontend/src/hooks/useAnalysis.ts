import { useCallback, useEffect, useState } from "react";
import { fetchAnalysis, fetchStatus, startAnalysis } from "../api/client";
import type { Analysis, RunStatus } from "../types/analysis";

export interface AnalysisState {
  analysis: Analysis | null;
  status: RunStatus | null;
  error: string | null;
  run: () => void;
}

export function useAnalysis(): AnalysisState {
  const [analysis, setAnalysis] = useState<Analysis | null>(null);
  const [status, setStatus] = useState<RunStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [a, s] = await Promise.all([fetchAnalysis(), fetchStatus()]);
      setAnalysis(a);
      setStatus(s);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (status?.state !== "running") return;
    const timer = setInterval(async () => {
      try {
        const s = await fetchStatus();
        setStatus(s);
        if (s.state !== "running") await load();
      } catch (e) {
        setError((e as Error).message);
      }
    }, 1000);
    return () => clearInterval(timer);
  }, [status?.state, load]);

  const run = useCallback(() => {
    startAnalysis()
      .then(setStatus)
      .catch((e: Error) => setError(e.message));
  }, []);

  return { analysis, status, error, run };
}
