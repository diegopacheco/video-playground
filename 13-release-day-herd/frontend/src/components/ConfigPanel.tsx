import type { SimConfig } from "../types/sim";

interface Props {
  config: SimConfig;
  running: boolean;
  onChange: (config: SimConfig) => void;
  onRun: () => void;
}

const fields: { key: keyof SimConfig; label: string }[] = [
  { key: "users", label: "Viewers at 00:00" },
  { key: "seed", label: "Seed" },
  { key: "press_window_ms", label: "Press spread (ms)" },
  { key: "api_rps", label: "Playback API capacity (rps)" },
  { key: "origin_rps", label: "Origin capacity (rps)" },
  { key: "admission_rps", label: "Waiting room admission (rps)" },
  { key: "edges", label: "CDN edges" },
  { key: "max_attempts", label: "Attempt budget per viewer" },
  { key: "backoff_base_ms", label: "Backoff base (ms)" },
  { key: "backoff_cap_ms", label: "Backoff cap (ms)" },
];

export function ConfigPanel({ config, running, onChange, onRun }: Props) {
  return (
    <section className="card config">
      <h2>Launch scenario</h2>
      <div className="fields">
        {fields.map((f) => (
          <label key={f.key}>
            <span>{f.label}</span>
            <input
              type="number"
              min={0}
              value={config[f.key]}
              onChange={(e) => onChange({ ...config, [f.key]: Number(e.target.value) })}
            />
          </label>
        ))}
      </div>
      <button className="primary" disabled={running} onClick={onRun}>
        {running ? "Simulating..." : "Run all strategies"}
      </button>
    </section>
  );
}
