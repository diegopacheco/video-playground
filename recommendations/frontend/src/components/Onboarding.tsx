import { useState } from "react";
import { api } from "../api/client";
import type { User } from "../types";
import { GenrePicker } from "./GenrePicker";

interface Props {
  genres: string[];
  onCreated: (user: User) => void;
  onCancel: () => void;
}

export function Onboarding({ genres, onCreated, onCancel }: Props) {
  const [name, setName] = useState("");
  const [picked, setPicked] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    try {
      onCreated(await api.createUser(name, picked));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="panel onboarding">
      <h2>Welcome! What do you like to watch?</h2>
      <p className="muted">
        New users have no history, so the first rows come from your genre picks (content similarity) blended with
        popularity.
      </p>
      <label className="field">
        <span>Name</span>
        <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Your name" />
      </label>
      <GenrePicker genres={genres} selected={picked} onChange={setPicked} />
      {error && <p className="error">{error}</p>}
      <div className="actions">
        <button className="primary" disabled={!name.trim()} onClick={submit}>
          Start watching ({picked.length} genres)
        </button>
        <button className="ghost" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </div>
  );
}
