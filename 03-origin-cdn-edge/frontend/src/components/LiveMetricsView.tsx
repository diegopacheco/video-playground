import { resetCounters } from "../api/client";
import type { LiveMetrics } from "../hooks/useLiveMetrics";
import { formatCount, formatPercent } from "../lib/format";
import { CacheUsage } from "./CacheUsage";
import { HitRatioGauge } from "./HitRatioGauge";
import { MetricCard } from "./MetricCard";
import { Sparkline } from "./Sparkline";

interface Props {
  live: LiveMetrics;
}

export function LiveMetricsView({ live }: Props) {
  const { edge, origin } = live;
  if (!edge) {
    return <div className="card">{live.error ? `Edge unreachable: ${live.error}` : "Loading metrics"}</div>;
  }
  return (
    <div className="stack">
      <div className="row">
        <HitRatioGauge ratio={edge.hit_ratio} label="Hit ratio (HIT + STALE)" />
        <HitRatioGauge ratio={edge.origin_offload} label="Origin offload" />
        <div className="grid">
          <MetricCard label="Requests" value={formatCount(edge.requests)} />
          <MetricCard label="Hits" value={formatCount(edge.hits)} tone="hit" />
          <MetricCard label="Misses" value={formatCount(edge.misses)} tone="miss" />
          <MetricCard label="Stale served" value={formatCount(edge.stale)} tone="stale" hint={`${formatCount(edge.revalidations)} background refreshes`} />
          <MetricCard label="Coalesced" value={formatCount(edge.coalesced)} hint="misses that waited on another fetch" />
          <MetricCard label="Origin fetches (edge)" value={formatCount(edge.origin_fetches)} hint={`${formatCount(edge.origin_errors)} errors, ${edge.in_flight} in flight`} />
          <MetricCard label="Origin requests (origin)" value={formatCount(origin?.requests ?? 0)} hint={origin ? `${origin.config.latency_ms} ms latency, max-age ${origin.config.max_age}s, swr ${origin.config.stale_while_revalidate}s` : "origin unreachable"} />
          <MetricCard label="Evictions" value={formatCount(edge.evictions)} />
        </div>
      </div>
      <CacheUsage bytes={edge.cache_bytes} capacity={edge.cache_capacity_bytes} entries={edge.cache_entries} evictions={edge.evictions} />
      <div className="row two">
        <Sparkline title="Hit ratio per second" values={live.ratioHistory} max={1} format={formatPercent} />
        <Sparkline title="Edge requests per second" values={live.rpsHistory} format={(v) => `${formatCount(v)} rps`} />
      </div>
      <div>
        <button type="button" className="secondary" onClick={() => void resetCounters()}>
          Reset counters
        </button>
      </div>
    </div>
  );
}
