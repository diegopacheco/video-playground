import { useCallback, useRef, useState } from "react";
import { fetchEdgeMetrics, fetchOriginStats } from "../api/client";
import { pushBounded, windowHitRatio, type Sample } from "../lib/history";
import type { EdgeMetrics, OriginStats } from "../types/metrics";
import { usePolling } from "./usePolling";

const HISTORY = 60;
const INTERVAL_MS = 1000;

export interface LiveMetrics {
  edge: EdgeMetrics | null;
  origin: OriginStats | null;
  ratioHistory: number[];
  rpsHistory: number[];
  error: string | null;
}

export function useLiveMetrics(): LiveMetrics {
  const previous = useRef<Sample | null>(null);
  const [ratioHistory, setRatioHistory] = useState<number[]>([]);
  const [rpsHistory, setRpsHistory] = useState<number[]>([]);

  const onEdge = useCallback((m: EdgeMetrics) => {
    const sample = { requests: m.requests, hits: m.hits, stale: m.stale };
    const ratio = windowHitRatio(previous.current, sample);
    const rps = previous.current && sample.requests >= previous.current.requests ? sample.requests - previous.current.requests : 0;
    previous.current = sample;
    if (ratio !== null) setRatioHistory((h) => pushBounded(h, ratio, HISTORY));
    setRpsHistory((h) => pushBounded(h, rps, HISTORY));
  }, []);

  const edge = usePolling(fetchEdgeMetrics, INTERVAL_MS, onEdge);
  const origin = usePolling(fetchOriginStats, INTERVAL_MS);

  return {
    edge: edge.data,
    origin: origin.data,
    ratioHistory,
    rpsHistory,
    error: edge.error ?? origin.error,
  };
}
