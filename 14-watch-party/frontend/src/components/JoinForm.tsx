import { useState, type FormEvent } from "react";

interface Props {
  onJoin: (room: string, name: string) => void;
}

export function JoinForm({ onJoin }: Props) {
  const [room, setRoom] = useState("movie-night");
  const [name, setName] = useState("");

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (room.trim() && name.trim()) onJoin(room.trim(), name.trim());
  };

  return (
    <form className="card join" onSubmit={submit}>
      <h2>Join a watch party</h2>
      <p className="muted">The first person in a room becomes the leader. Everyone else follows the leader's clock.</p>
      <label>
        Your name
        <input name="name" value={name} maxLength={32} onChange={(e) => setName(e.target.value)} autoFocus />
      </label>
      <label>
        Room
        <input name="room" value={room} maxLength={32} onChange={(e) => setRoom(e.target.value)} />
      </label>
      <button type="submit" disabled={!room.trim() || !name.trim()}>
        Join room
      </button>
    </form>
  );
}
