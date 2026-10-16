import { SCENARIOS } from "../scenarios";
import type { Scenario } from "../types";

interface Props {
  active: string;
  onRun: (scenario: Scenario) => void;
}

export function ScenarioPanel({ active, onRun }: Props) {
  return (
    <section className="card">
      <h2>Scenarios</h2>
      <div className="scenarios">
        {SCENARIOS.map((s) => (
          <button key={s.id} className={s.id === active ? "scenario active" : "scenario"} onClick={() => onRun(s)}>
            <strong>{s.title}</strong>
            <span>{s.description}</span>
            <em>expect: {s.expect}</em>
          </button>
        ))}
      </div>
    </section>
  );
}
