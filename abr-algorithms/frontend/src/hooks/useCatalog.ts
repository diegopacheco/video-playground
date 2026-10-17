import { useEffect, useState } from "react";
import { fetchAlgorithms, fetchTraces } from "../api/client";
import type { AlgorithmInfo, TraceSummary } from "../types/abr";

export interface Catalog {
  traces: TraceSummary[];
  algorithms: AlgorithmInfo[];
  error: string | null;
}

export function useCatalog(): Catalog {
  const [catalog, setCatalog] = useState<Catalog>({ traces: [], algorithms: [], error: null });
  useEffect(() => {
    Promise.all([fetchTraces(), fetchAlgorithms()])
      .then(([traces, algorithms]) => setCatalog({ traces, algorithms, error: null }))
      .catch((e: unknown) => setCatalog({ traces: [], algorithms: [], error: e instanceof Error ? e.message : String(e) }));
  }, []);
  return catalog;
}
