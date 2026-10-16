import { formatBytes, formatCount } from "../lib/format";

interface Props {
  bytes: number;
  capacity: number;
  entries: number;
  evictions: number;
}

export function CacheUsage({ bytes, capacity, entries, evictions }: Props) {
  const pct = capacity > 0 ? Math.min(bytes / capacity, 1) * 100 : 0;
  return (
    <div className="card usage">
      <div className="spark-head">
        <span className="metric-label">LRU byte budget</span>
        <span className="spark-last">
          {formatBytes(bytes)} / {formatBytes(capacity)}
        </span>
      </div>
      <div className="bar">
        <div className="bar-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="metric-hint">
        {formatCount(entries)} objects cached, {formatCount(evictions)} evicted
      </div>
    </div>
  );
}
