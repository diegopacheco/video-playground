import type { AuditEvent } from "../types";

export function AuditPanel({ events }: { events: AuditEvent[] }) {
  return (
    <section className="card">
      <h2>Server decisions</h2>
      <table className="audit">
        <thead>
          <tr>
            <th>time</th>
            <th>kind</th>
            <th>outcome</th>
            <th>detail</th>
          </tr>
        </thead>
        <tbody>
          {events.slice(0, 12).map((e, i) => (
            <tr key={`${e.at}-${i}`}>
              <td>{new Date(e.at * 1000).toLocaleTimeString()}</td>
              <td>{e.kind}</td>
              <td className={e.outcome === "granted" ? "ok" : "error"}>{e.outcome}</td>
              <td className="mono small">{e.detail}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
