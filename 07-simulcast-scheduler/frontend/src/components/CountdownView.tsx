import { useState } from "react";
import { api } from "../api/client";
import type { Board } from "../types";
import { RegionCard } from "./RegionCard";

interface Props {
  board: Board;
  onChange: () => void;
}

export function CountdownView({ board, onChange }: Props) {
  const [selected, setSelected] = useState(board.episodes[0]?.id ?? "");
  const episode = board.episodes.find((e) => e.id === selected) ?? board.episodes[0];
  if (!episode) return <p>No episodes scheduled.</p>;

  return (
    <section className="panel">
      <div className="row">
        <select value={episode.id} onChange={(e) => setSelected(e.target.value)}>
          {board.episodes.map((e) => (
            <option key={e.id} value={e.id}>
              {e.title}
            </option>
          ))}
        </select>
        <span className="muted">global release {episode.release_utc.replace("T", " ").slice(0, 19)} UTC</span>
        <button onClick={() => api.release(episode.id, 15).then(onChange)}>Release in 15s</button>
      </div>
      <div className="grid">
        {episode.regions.map((r) => (
          <RegionCard key={r.country} region={r} />
        ))}
      </div>
    </section>
  );
}
