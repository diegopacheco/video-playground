import { useEffect, useRef, useState } from "react";

export interface Polled<T> {
  data: T | null;
  error: string | null;
}

export function usePolling<T>(load: () => Promise<T>, intervalMs: number, onData?: (value: T) => void): Polled<T> {
  const [state, setState] = useState<Polled<T>>({ data: null, error: null });
  const onDataRef = useRef(onData);
  onDataRef.current = onData;

  useEffect(() => {
    let active = true;
    const tick = async () => {
      try {
        const value = await load();
        if (!active) return;
        setState({ data: value, error: null });
        onDataRef.current?.(value);
      } catch (err) {
        if (active) setState((prev) => ({ data: prev.data, error: err instanceof Error ? err.message : String(err) }));
      }
    };
    void tick();
    const timer = setInterval(() => void tick(), intervalMs);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [load, intervalMs]);

  return state;
}
