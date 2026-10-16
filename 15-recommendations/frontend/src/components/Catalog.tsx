import { useState } from "react";
import { api } from "../api/client";
import { useAsync } from "../hooks/useAsync";
import { GenrePicker } from "./GenrePicker";
import { TitleCard } from "./TitleCard";

interface Props {
  genres: string[];
  version: number;
  onOpen: (id: number) => void;
  onCreated: (id: number) => void;
}

export function Catalog({ genres, version, onOpen, onCreated }: Props) {
  const { data } = useAsync(() => api.titles(), [version]);
  const [filter, setFilter] = useState<string>("");
  const [name, setName] = useState("");
  const [picked, setPicked] = useState<string[]>([]);
  const [tags, setTags] = useState("");
  const [synopsis, setSynopsis] = useState("");
  const [error, setError] = useState<string | null>(null);

  const titles = (data ?? [])
    .filter((t) => !filter || t.title.genres.includes(filter))
    .sort((a, b) => b.watches - a.watches);

  const create = async () => {
    try {
      const title = await api.createTitle({
        name,
        genres: picked,
        tags: tags.split(",").map((t) => t.trim()).filter(Boolean),
        synopsis,
        year: new Date().getFullYear(),
      });
      setName("");
      setPicked([]);
      setTags("");
      setSynopsis("");
      setError(null);
      onCreated(title.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="catalog">
      <div className="panel">
        <h2>Add a new title</h2>
        <p className="muted">
          A brand-new title has zero watches, so collaborative filtering cannot see it. Its embedding is computed
          from genres, tags and synopsis and stored in pgvector, so it shows up in similar-title rows right away.
        </p>
        <label className="field">
          <span>Name</span>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Title name" />
        </label>
        <GenrePicker genres={genres} selected={picked} onChange={setPicked} />
        <label className="field">
          <span>Tags</span>
          <input value={tags} onChange={(e) => setTags(e.target.value)} placeholder="giant robots, space colony" />
        </label>
        <label className="field">
          <span>Synopsis</span>
          <textarea value={synopsis} onChange={(e) => setSynopsis(e.target.value)} rows={2} />
        </label>
        {error && <p className="error">{error}</p>}
        <button className="primary" disabled={!name.trim() || picked.length === 0} onClick={create}>
          Add title
        </button>
      </div>
      <div className="catalog-head">
        <h2>Catalog ({titles.length})</h2>
        <select value={filter} onChange={(e) => setFilter(e.target.value)}>
          <option value="">All genres</option>
          {genres.map((g) => (
            <option key={g} value={g}>
              {g}
            </option>
          ))}
        </select>
      </div>
      <div className="grid">
        {titles.map((t) => (
          <TitleCard key={t.title.id} title={t.title} subtitle={`${t.watches} watches · ${t.title.genres.join(" · ")}`} onOpen={onOpen} />
        ))}
      </div>
    </div>
  );
}
