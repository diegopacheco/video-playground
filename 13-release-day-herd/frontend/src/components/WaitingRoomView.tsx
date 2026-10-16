import { useWaitingRoom } from "../hooks/useWaitingRoom";

export function WaitingRoomView() {
  const { viewers, stats, error, join, reset } = useWaitingRoom();
  return (
    <div className="stack">
      <section className="card">
        <h2>Live waiting room</h2>
        <p className="muted">
          Real axum service. Each join gets a ticket with position and ETA; tokens are released at the admission rate and
          the play endpoint refuses anyone without one.
        </p>
        <div className="row">
          <button className="primary" onClick={() => join(1)}>
            Join as 1 viewer
          </button>
          <button onClick={() => join(50)}>Join 50 viewers</button>
          <button onClick={() => join(300)}>Join 300 viewers</button>
          <button onClick={reset}>Reset room</button>
        </div>
        {error && <p className="error">{error}</p>}
      </section>
      {stats && (
        <section className="card stats">
          <Stat label="Admission rate" value={`${stats.rate_per_s}/s`} />
          <Stat label="Joined" value={stats.joined} />
          <Stat label="Waiting" value={stats.waiting} />
          <Stat label="Admitted" value={stats.admitted} />
          <Stat label="Played" value={stats.played} />
          <Stat
            label="Max admitted in any 1 s"
            value={stats.max_admitted_in_any_second}
            good={stats.max_admitted_in_any_second <= stats.rate_per_s}
          />
        </section>
      )}
      {viewers.length > 0 && (
        <section className="card">
          <h2>Your viewers</h2>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Ticket</th>
                  <th>ETA at join</th>
                  <th>Position</th>
                  <th>ETA now</th>
                  <th>State</th>
                </tr>
              </thead>
              <tbody>
                {viewers.slice(0, 12).map((v) => (
                  <tr key={v.ticket}>
                    <td>#{v.ticket}</td>
                    <td>{v.joinedEtaMs} ms</td>
                    <td>{v.position}</td>
                    <td>{v.etaMs} ms</td>
                    <td className={v.manifest ? "good" : ""}>{v.manifest ? `playing ${v.manifest}` : "waiting"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {viewers.length > 12 && <p className="muted">showing newest 12 of {viewers.length}</p>}
        </section>
      )}
    </div>
  );
}

function Stat({ label, value, good }: { label: string; value: string | number; good?: boolean }) {
  return (
    <div className="stat">
      <span>{label}</span>
      <strong className={good ? "good" : ""}>{value}</strong>
    </div>
  );
}
