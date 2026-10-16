import type { UserProfile } from "../types";

interface Props {
  users: UserProfile[];
  selected: string;
  onSelect: (id: string) => void;
}

export function UserPicker({ users, selected, onSelect }: Props) {
  return (
    <div className="users">
      {users.map((u) => (
        <button key={u.id} className={`user ${u.id === selected ? "active" : ""}`} onClick={() => onSelect(u.id)}>
          <strong>{u.name}</strong>
          <span className="tag">{u.region.toUpperCase()}</span>
          <span className={`tag ${u.tier}`}>{u.tier}</span>
        </button>
      ))}
    </div>
  );
}
