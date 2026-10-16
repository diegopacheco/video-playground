import { useCallback, useEffect, useState } from "react";

export function usePoll<T>(load: () => Promise<T>, intervalMs: number) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    load()
      .then((d) => {
        setData(d);
        setError(null);
      })
      .catch((e: unknown) => setError(String(e)));
  }, [load]);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, intervalMs);
    return () => clearInterval(id);
  }, [refresh, intervalMs]);

  return { data, error, refresh };
}
