import { useEffect, useState } from "react";
import { fetchStitch } from "../api/client";
import type { StitchView } from "../types";

export function useStitch(user: string, session: string): { view: StitchView | null; error: string | null } {
  const [view, setView] = useState<StitchView | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setView(null);
    fetchStitch(user, session)
      .then((v) => {
        if (active) setView(v);
      })
      .catch((e: unknown) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [user, session]);

  return { view, error };
}
