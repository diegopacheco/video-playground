import { deviceKey } from "../hooks/useDevices";
import type { TimedSnapshot } from "../hooks/useSnapshot";
import { fraction, remainingMs, seconds } from "../lib/time";
import type { DeviceState } from "../types";

interface Props {
  data: TimedSnapshot | null;
  devices: Record<string, DeviceState>;
  now: number;
}

export function LeaseTable({ data, devices, now }: Props) {
  if (!data) {
    return <section className="card">Backend unreachable</section>;
  }
  const { snapshot, fetchedAt } = data;
  const used = snapshot.leases.length;
  const limit = snapshot.account.limit;
  return (
    <section className="card">
      <div className="card-head">
        <h2>Leases in Redis</h2>
        <span className={used >= limit ? "slots full" : "slots"}>
          {used} / {limit} slots
        </span>
      </div>
      <table>
        <thead>
          <tr>
            <th>Device</th>
            <th>Started</th>
            <th>Expires in</th>
            <th>Client</th>
          </tr>
        </thead>
        <tbody>
          {snapshot.leases.length === 0 && (
            <tr>
              <td colSpan={4} className="muted">
                no active leases
              </td>
            </tr>
          )}
          {snapshot.leases.map((lease) => {
            const left = remainingMs(lease.expires_at_ms, snapshot.now_ms, fetchedAt, now);
            const ago = Math.max(0, snapshot.now_ms + (now - fetchedAt) - lease.started_at_ms);
            const client = devices[deviceKey(snapshot.account.id, lease.device)]?.status ?? "idle";
            const zombie = client !== "playing";
            return (
              <tr key={lease.device} className={zombie ? "zombie" : ""}>
                <td>{lease.device}</td>
                <td>{seconds(ago)} ago</td>
                <td>
                  <div className="countdown">
                    <div className="bar">
                      <div
                        className={zombie ? "fill fill-zombie" : "fill"}
                        style={{ width: `${fraction(left, snapshot.ttl_ms) * 100}%` }}
                      />
                    </div>
                    <span>{seconds(left)}</span>
                  </div>
                </td>
                <td>{zombie ? "no heartbeat" : "heartbeating"}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </section>
  );
}
