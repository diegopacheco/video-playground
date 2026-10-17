import type { Device } from "../hooks/useDevice";
import { TITLES, findTitle, formatTime } from "../lib/playback";

const SKEWS: { label: string; value: number }[] = [
  { label: "clock in sync", value: 0 },
  { label: "clock 2 min behind", value: -120_000 },
  { label: "clock 2 min ahead", value: 120_000 },
];

interface Props {
  label: string;
  deviceId: string;
  device: Device;
}

export function DevicePanel({ label, deviceId, device }: Props) {
  const { state } = device;
  const title = findTitle(state.titleId);
  const pct = (100 * state.position) / title.duration;
  return (
    <section className="device" data-testid={`device-${deviceId}`}>
      <header>
        <h2>{label}</h2>
        <code>{deviceId}</code>
      </header>
      <div className="screen" style={{ background: `hsl(${title.hue} 70% 92%)`, borderColor: `hsl(${title.hue} 50% 70%)` }}>
        <strong>{title.name}</strong>
        <span>{title.episode}</span>
        <span className="clock">
          {formatTime(state.position)} / {formatTime(title.duration)}
        </span>
        <span className={state.playing ? "live on" : "live"}>{state.playing ? "playing" : "paused"}</span>
      </div>
      <div className="bar">
        <div style={{ width: `${pct}%`, background: `hsl(${title.hue} 60% 55%)` }} />
      </div>
      <div className="controls">
        <select value={state.titleId} onChange={(e) => device.selectTitle(e.target.value)} aria-label={`${label} title`}>
          {TITLES.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
        <button onClick={device.togglePlay}>{state.playing ? "Pause" : "Play"}</button>
        <button onClick={() => device.seek(-30)}>-30s</button>
        <button onClick={() => device.seek(60)}>+60s</button>
      </div>
      <div className="controls">
        <select value={state.skewMs} onChange={(e) => device.setSkew(Number(e.target.value))} aria-label={`${label} clock`}>
          {SKEWS.map((s) => (
            <option key={s.value} value={s.value}>
              {s.label}
            </option>
          ))}
        </select>
        <span className="meta">heartbeats sent {state.sent}</span>
        {state.lastResult && <span className={`badge ${state.lastResult}`}>{state.lastResult}</span>}
      </div>
    </section>
  );
}
