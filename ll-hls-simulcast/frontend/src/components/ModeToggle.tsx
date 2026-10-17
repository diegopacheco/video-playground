import { MODES, MODE_LABEL } from "../api/streams";
import type { Mode } from "../types/latency";

interface Props {
  mode: Mode;
  onChange: (mode: Mode) => void;
}

export function ModeToggle({ mode, onChange }: Props) {
  return (
    <div className="toggle" role="tablist">
      {MODES.map((m) => (
        <button
          key={m}
          role="tab"
          data-mode={m}
          aria-selected={m === mode}
          className={m === mode ? "active" : ""}
          onClick={() => onChange(m)}
        >
          {MODE_LABEL[m]}
        </button>
      ))}
    </div>
  );
}
