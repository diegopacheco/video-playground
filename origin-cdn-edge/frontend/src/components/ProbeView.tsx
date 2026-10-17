import { useState } from "react";
import { fetchOriginKeyRequests, probeBurst } from "../api/client";
import { countStatuses } from "../lib/history";
import type { ProbeResult } from "../types/metrics";

export function ProbeView() {
  const [key, setKey] = useState("seg-00001.ts");
  const [concurrency, setConcurrency] = useState(100);
  const [result, setResult] = useState<ProbeResult | null>(null);
  const [busy, setBusy] = useState(false);

  const run = async () => {
    setBusy(true);
    try {
      const before = await fetchOriginKeyRequests(key);
      const hits = await probeBurst(key, concurrency);
      const after = await fetchOriginKeyRequests(key);
      setResult({ key, hits, originRequestsDelta: after - before });
    } catch {
      setResult({ key, hits: [], originRequestsDelta: -1 });
    } finally {
      setBusy(false);
    }
  };

  const counts = result ? countStatuses(result.hits) : null;
  const coalesced = result ? result.hits.filter((h) => h.coalesced).length : 0;
  const slowest = result ? Math.max(0, ...result.hits.map((h) => h.millis)) : 0;

  return (
    <div className="stack">
      <div className="card probe-form">
        <label>
          Key
          <input value={key} onChange={(e) => setKey(e.target.value)} />
        </label>
        <label>
          Concurrent requests
          <input type="number" min={1} max={1000} value={concurrency} onChange={(e) => setConcurrency(Math.max(1, Math.min(1000, Number(e.target.value) || 1)))} />
        </label>
        <button type="button" disabled={busy} onClick={() => void run()}>
          {busy ? "Firing" : "Fire burst"}
        </button>
        <button type="button" className="secondary" disabled={busy} onClick={() => setKey(`seg-${Date.now()}.ts`)}>
          New cold key
        </button>
      </div>
      {result && counts ? (
        <div className="card">
          <h3>
            {result.hits.length} requests for <code>{result.key}</code>
          </h3>
          <div className="pills">
            <span className="pill hit">HIT {counts.HIT}</span>
            <span className="pill miss">MISS {counts.MISS}</span>
            <span className="pill stale">STALE {counts.STALE}</span>
            <span className="pill">coalesced {coalesced}</span>
            <span className="pill">ERROR {counts.ERROR}</span>
          </div>
          <p className="verdict">
            Origin received <strong>{result.originRequestsDelta}</strong> request(s) for <code>{result.key}</code> during this burst. Slowest response {slowest.toFixed(0)} ms.
          </p>
          <div className="dots">
            {result.hits.map((h, i) => (
              <span key={i} className={`dot ${h.status.toLowerCase()}${h.coalesced ? " coalesced" : ""}`} title={`${h.status} ${h.millis.toFixed(0)} ms`} />
            ))}
          </div>
        </div>
      ) : (
        <div className="card muted">The edge fires N concurrent requests for the key server side. On a cold key every request misses, but the origin should be hit exactly once.</div>
      )}
    </div>
  );
}
