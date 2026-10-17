import type { Beacon } from "../types";

interface Props {
  beacons: Beacon[];
}

export function BeaconLog({ beacons }: Props) {
  return (
    <section className="beacons">
      <header>
        <h3>Beacon log</h3>
        <span className="counts">{beacons.length} recorded</span>
      </header>
      {beacons.length === 0 ? (
        <p className="empty">No beacons yet. They fire when the player fetches tracked ad segments.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>time</th>
              <th>break</th>
              <th>creative</th>
              <th>event</th>
            </tr>
          </thead>
          <tbody>
            {[...beacons].reverse().map((b) => (
              <tr key={b.seq}>
                <td>{b.seq}</td>
                <td>{new Date(b.atMs).toLocaleTimeString()}</td>
                <td>{b.breakId}</td>
                <td>
                  <span className={`chip c-${b.creative}`}>{b.creative}</span>
                </td>
                <td className={`ev ${b.event}`}>{b.event}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
