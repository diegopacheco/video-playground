import { useState } from "react";
import type { Field, Hit } from "../types/search";
import { Highlighted } from "./Highlighted";
import { ScoreBreakdown } from "./ScoreBreakdown";

const LABELS: Record<Field, string> = { romaji: "romaji", english: "english", japanese: "日本語" };

export function ResultCard({ hit, rank }: { hit: Hit; rank: number }) {
  const [open, setOpen] = useState(rank === 1);
  return (
    <li className="card">
      <button className="card-head" onClick={() => setOpen(!open)} aria-expanded={open}>
        <span className="rank">{rank}</span>
        <span className="titles">
          {hit.highlights.map((h) => (
            <span key={h.field} className={`title title-${h.field} ${h.field === hit.breakdown.field ? "best" : ""}`}>
              <small>{LABELS[h.field]}</small>
              <Highlighted segments={h.segments} />
            </span>
          ))}
        </span>
        <span className="meta">
          <strong>{hit.score.toFixed(3)}</strong>
          <small>{hit.year}</small>
        </span>
      </button>
      {open && <ScoreBreakdown breakdown={hit.breakdown} />}
    </li>
  );
}
