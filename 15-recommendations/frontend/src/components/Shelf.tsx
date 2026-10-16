import type { Item } from "../types";
import { TitleCard } from "./TitleCard";

interface Props {
  label: string;
  hint?: string;
  items: Item[];
  empty?: string;
  onOpen: (id: number) => void;
}

export function Shelf({ label, hint, items, empty, onOpen }: Props) {
  return (
    <section className="shelf">
      <div className="shelf-head">
        <h3>{label}</h3>
        {hint && <span className="shelf-hint">{hint}</span>}
      </div>
      {items.length === 0 ? (
        <p className="muted">{empty ?? "Nothing here yet."}</p>
      ) : (
        <div className="shelf-row">
          {items.map((i) => (
            <TitleCard key={i.title.id} title={i.title} score={i.score} onOpen={onOpen} />
          ))}
        </div>
      )}
    </section>
  );
}
