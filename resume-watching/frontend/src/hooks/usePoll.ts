import { useCallback, useEffect, useState } from "react";

export function usePoll<T>(load: () => Promise<T>, everyMs: number): { data: T | null; error: string | null; refresh: () => void } {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    load()
      .then((d) => {
        setData(d);
        setError(null);
      })
      .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)));
  }, [load]);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, everyMs);
    return () => clearInterval(id);
  }, [refresh, everyMs]);

  return { data, error, refresh };
}
