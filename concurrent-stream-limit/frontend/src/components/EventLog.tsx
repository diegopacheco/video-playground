import type { LogEntry } from "../types";

export function EventLog({ entries }: { entries: LogEntry[] }) {
  return (
    <section className="card">
      <div className="card-head">
        <h2>Events</h2>
      </div>
      <ul className="log">
        {entries.length === 0 && <li className="muted">press Start on a device</li>}
        {entries.map((e, i) => (
          <li key={`${e.at}-${i}`} className={`log-${e.tone}`}>
            <time>{new Date(e.at).toLocaleTimeString()}</time>
            {e.text}
          </li>
        ))}
      </ul>
    </section>
  );
}
