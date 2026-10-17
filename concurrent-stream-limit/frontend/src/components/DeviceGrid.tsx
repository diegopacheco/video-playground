import { deviceKey } from "../hooks/useDevices";
import type { DeviceState } from "../types";

export const DEVICES = ["living-room-tv", "bedroom-tv", "phone", "tablet", "laptop"];

interface Props {
  account: string;
  devices: Record<string, DeviceState>;
  onStart: (device: string) => void;
  onStop: (device: string) => void;
  onCrash: (device: string) => void;
}

const LABELS: Record<DeviceState["status"], string> = {
  idle: "idle",
  playing: "playing",
  rejected: "rejected",
  kicked: "kicked",
  crashed: "crashed",
};

export function DeviceGrid({ account, devices, onStart, onStop, onCrash }: Props) {
  return (
    <section className="devices">
      {DEVICES.map((name) => {
        const state = devices[deviceKey(account, name)] ?? { status: "idle", beats: 0 };
        const playing = state.status === "playing";
        return (
          <article key={name} className={`device status-${state.status}`}>
            <header>
              <strong>{name}</strong>
              <span className={`pill pill-${state.status}`}>{LABELS[state.status]}</span>
            </header>
            <div className="screen">{playing ? <span>heartbeats {state.beats}</span> : <span>{LABELS[state.status]}</span>}</div>
            <div className="actions">
              <button className="btn start" disabled={playing} onClick={() => onStart(name)}>
                Start
              </button>
              <button className="btn stop" onClick={() => onStop(name)}>
                Stop
              </button>
              <button className="btn crash" disabled={!playing} onClick={() => onCrash(name)}>
                Crash
              </button>
            </div>
          </article>
        );
      })}
    </section>
  );
}
