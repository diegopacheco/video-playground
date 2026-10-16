import { useEffect, useState } from "react";
import { search } from "../api/client";
import type { SearchResponse } from "../types/search";

const DEBOUNCE_MS = 60;

export interface SearchState {
  response: SearchResponse | null;
  error: string | null;
  loading: boolean;
}

export function useSearch(query: string, limit: number): SearchState {
  const [state, setState] = useState<SearchState>({ response: null, error: null, loading: false });

  useEffect(() => {
    if (query.trim() === "") {
      setState({ response: null, error: null, loading: false });
      return;
    }
    const controller = new AbortController();
    setState((s) => ({ ...s, loading: true }));
    const timer = setTimeout(() => {
      search(query, limit, controller.signal)
        .then((response) => setState({ response, error: null, loading: false }))
        .catch((e: unknown) => {
          if (controller.signal.aborted) return;
          setState({ response: null, error: e instanceof Error ? e.message : String(e), loading: false });
        });
    }, DEBOUNCE_MS);
    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [query, limit]);

  return state;
}
