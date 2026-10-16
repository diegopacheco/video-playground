import type { Member } from "../types/protocol";

interface Props {
  members: Member[];
  leader: number;
  you: number | null;
}

export function Members({ members, leader, you }: Props) {
  return (
    <section className="card">
      <h3>Members ({members.length})</h3>
      <ul className="members">
        {members.map((m) => (
          <li key={m.id}>
            <span className="name">{m.name}</span>
            {m.id === leader && <span className="badge leader">leader</span>}
            {m.id === you && <span className="badge you">you</span>}
          </li>
        ))}
      </ul>
    </section>
  );
}
