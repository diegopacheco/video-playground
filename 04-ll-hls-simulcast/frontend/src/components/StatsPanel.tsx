import { MODES, MODE_LABEL } from "../api/streams";
import { formatMs, summarize } from "../lib/stats";
import type { History, Mode } from "../types/latency";

interface Props {
  history: History;
  active: Mode;
}

export function StatsPanel({ history, active }: Props) {
  return (
    <table className="stats">
      <thead>
        <tr>
          <th>Mode</th>
          <th>Now</th>
          <th>Min</th>
          <th>Avg</th>
          <th>p50</th>
          <th>p95</th>
          <th>Samples</th>
        </tr>
      </thead>
      <tbody>
        {MODES.map((mode) => {
          const s = summarize(history[mode]);
          return (
            <tr key={mode} data-mode={mode} className={mode === active ? "active" : ""}>
              <td>
                <span className={`swatch ${mode}`} />
                {MODE_LABEL[mode]}
              </td>
              <td>{formatMs(s.current)}</td>
              <td>{formatMs(s.min)}</td>
              <td>{formatMs(s.avg)}</td>
              <td>{formatMs(s.p50)}</td>
              <td>{formatMs(s.p95)}</td>
              <td>{s.count}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
