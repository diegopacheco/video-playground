import type { Title } from "../types";
import { initials, posterColors } from "../lib/display";

interface Props {
  title: Title;
  score?: number;
  subtitle?: string;
  onOpen: (id: number) => void;
}

export function TitleCard({ title, score, subtitle, onOpen }: Props) {
  const [from, to] = posterColors(title.id);
  return (
    <button className="card" onClick={() => onOpen(title.id)} title={title.synopsis}>
      <div className="poster" style={{ background: `linear-gradient(135deg, ${from}, ${to})` }}>
        <span className="poster-initials">{initials(title.name)}</span>
        {title.is_new && <span className="badge new">NEW</span>}
        {score !== undefined && <span className="badge score">{score.toFixed(2)}</span>}
      </div>
      <div className="card-name">{title.name}</div>
      <div className="card-meta">{subtitle ?? title.genres.join(" · ")}</div>
    </button>
  );
}
