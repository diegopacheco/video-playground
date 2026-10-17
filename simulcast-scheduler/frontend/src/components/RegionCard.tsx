import { formatCountdown } from "../lib/time";
import type { Entitlement } from "../types";

export function RegionCard({ region }: { region: Entitlement }) {
  const local = region.local_release;
  return (
    <div className={`card ${region.status}`}>
      <div className="card-head">
        <strong>{region.country}</strong>
        <span className={`chip ${region.status}`}>{region.status.replace("_", " ")}</span>
      </div>
      <div className="countdown">
        {region.status === "locked" && formatCountdown(region.countdown_ms ?? 0)}
        {region.status === "available" && "Watch now"}
        {region.status === "geo_blocked" && (region.reason ?? "").replace("_", " ")}
      </div>
      <div className="muted small">{local?.timezone ?? "no release in this region"}</div>
      {local && (
        <div className="small">
          {local.local} {local.abbrev} ({local.utc_offset})
        </div>
      )}
      {region.available_until && (
        <div className="small muted">until {region.available_until.replace("T", " ").slice(0, 16)} UTC</div>
      )}
    </div>
  );
}
