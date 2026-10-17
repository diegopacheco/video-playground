import type { LogEntry } from "../types";

export function EventLog({ logs }: { logs: LogEntry[] }) {
  return (
    <section className="card">
      <h2>Client events</h2>
      <ul className="log">
        {logs.map((l) => (
          <li key={l.id} className={l.level}>
            {l.text}
          </li>
        ))}
      </ul>
    </section>
  );
}
