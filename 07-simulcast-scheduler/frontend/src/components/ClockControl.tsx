import { useState } from "react";
import { api } from "../api/client";
import { formatOffset, nextUnlockMs } from "../lib/time";
import type { Board } from "../types";

interface Props {
  board: Board;
  onChange: () => void;
}

const steps: Array<[string, number]> = [
  ["-1h", -3_600_000],
  ["-1m", -60_000],
  ["-10s", -10_000],
  ["+10s", 10_000],
  ["+1m", 60_000],
  ["+1h", 3_600_000],
  ["+1d", 86_400_000],
];

export function ClockControl({ board, onChange }: Props) {
  const [custom, setCustom] = useState("0");
  const apply = (offset: number) => api.setOffset(offset).then(onChange);
  const next = nextUnlockMs(board);
  const now = new Date(board.server_now);

  return (
    <section className="panel clock">
      <div className="clock-now">
        <span className="muted">Simulated server time (UTC)</span>
        <strong>{now.toISOString().replace("T", " ").slice(0, 19)}</strong>
        <span className={board.offset_ms === 0 ? "pill neutral" : "pill warn"}>offset {formatOffset(board.offset_ms)}</span>
      </div>
      <div className="clock-buttons">
        {steps.map(([label, delta]) => (
          <button key={label} onClick={() => apply(board.offset_ms + delta)}>
            {label}
          </button>
        ))}
        <button disabled={next === null} onClick={() => next !== null && apply(board.offset_ms + next - 10_000)}>
          Jump to T-10s of next unlock
        </button>
        <button className="ghost" onClick={() => apply(0)}>
          Reset
        </button>
        <label className="custom">
          offset seconds
          <input value={custom} onChange={(e) => setCustom(e.target.value)} inputMode="numeric" />
          <button onClick={() => apply(Number(custom) * 1000 || 0)}>Set</button>
        </label>
      </div>
    </section>
  );
}
