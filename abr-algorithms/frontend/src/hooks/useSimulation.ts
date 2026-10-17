import { useCallback, useState } from "react";
import { simulate } from "../api/client";
import type { SimulationReport } from "../types/abr";

export function useSimulation() {
  const [report, setReport] = useState<SimulationReport | null>(null);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async (trace: string, algorithms: string[]) => {
    setRunning(true);
    setError(null);
    try {
      setReport(await simulate(trace, algorithms));
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setRunning(false);
    }
  }, []);

  return { report, running, error, run };
}
