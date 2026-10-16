import { findTitle, formatTime } from "../lib/playback";
import type { RowItem } from "../types";

interface Props {
  items: RowItem[];
}

function ago(ts: number): string {
  const secs = Math.round((Date.now() - ts) / 1000);
  if (secs < 0) return `${-secs}s in the future`;
  return secs < 60 ? `${secs}s ago` : `${Math.floor(secs / 60)}m ago`;
}

export function ContinueRow({ items }: Props) {
  if (items.length === 0) {
    return <p className="empty">Nothing to continue. Press Play on a device.</p>;
  }
  return (
    <ol className="row" data-testid="continue-row">
      {items.map((item, i) => {
        const title = findTitle(item.title_id);
        return (
          <li key={item.title_id} className="card" style={{ borderTopColor: `hsl(${title.hue} 55% 60%)` }}>
            <div className="poster" style={{ background: `hsl(${title.hue} 70% 90%)` }}>
              <span className="rank">{i + 1}</span>
              <strong>{title.name}</strong>
              <span>{item.episode_id}</span>
            </div>
            <div className="bar">
              <div style={{ width: `${item.progress * 100}%`, background: `hsl(${title.hue} 60% 55%)` }} />
            </div>
            <div className="card-meta">
              <span>resume at {formatTime(item.position_secs)}</span>
              <span>{Math.round(item.progress * 100)}%</span>
            </div>
            <div className="card-meta">
              <code>{item.device_id}</code>
              <span>{ago(item.client_ts_ms)}</span>
            </div>
            <span className={`badge ${item.source}`}>{item.source === "buffer" ? "in buffer" : "persisted"}</span>
          </li>
        );
      })}
    </ol>
  );
}
