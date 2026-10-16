import type { AlgorithmInfo, TraceSummary } from "../types/abr";
import { algorithmColor } from "../lib/colors";

interface Props {
  traces: TraceSummary[];
  algorithms: AlgorithmInfo[];
  trace: string;
  selected: string[];
  running: boolean;
  onTrace: (id: string) => void;
  onToggle: (id: string) => void;
  onRun: () => void;
}

export function Controls({ traces, algorithms, trace, selected, running, onTrace, onToggle, onRun }: Props) {
  const current = traces.find((t) => t.id === trace);
  return (
    <section className="panel controls">
      <div className="field">
        <label htmlFor="trace">Network trace</label>
        <select id="trace" value={trace} onChange={(e) => onTrace(e.target.value)}>
          {traces.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
        {current && <p className="hint">{current.description}</p>}
      </div>
      <div className="field">
        <span className="label">Algorithms</span>
        <div className="algos">
          {algorithms.map((a) => (
            <label key={a.id} className="algo" title={a.description}>
              <input type="checkbox" checked={selected.includes(a.id)} onChange={() => onToggle(a.id)} />
              <span className="dot" style={{ background: algorithmColor(a.id) }} />
              <span>
                <strong>{a.name}</strong>
                <small>{a.description}</small>
              </span>
            </label>
          ))}
        </div>
      </div>
      <button type="button" onClick={onRun} disabled={running || selected.length === 0}>
        {running ? "Simulating..." : "Run simulation"}
      </button>
    </section>
  );
}
