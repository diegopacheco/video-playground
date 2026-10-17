import { useEffect, useState } from "react";
import { fetchTracks } from "../api/client";
import type { TrackInfo } from "../types";

export function useTracks(): { tracks: TrackInfo[]; error: string | null } {
  const [tracks, setTracks] = useState<TrackInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    fetchTracks().then(setTracks, (e: unknown) => setError(String(e)));
  }, []);
  return { tracks, error };
}
