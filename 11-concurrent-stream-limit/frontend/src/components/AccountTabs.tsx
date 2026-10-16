import type { Account } from "../types";

interface Props {
  accounts: Account[];
  selected: string;
  onSelect: (id: string) => void;
}

export function AccountTabs({ accounts, selected, onSelect }: Props) {
  return (
    <nav className="tabs">
      {accounts.map((a) => (
        <button
          key={a.id}
          className={a.id === selected ? "tab active" : "tab"}
          onClick={() => onSelect(a.id)}
        >
          <span className="tab-name">{a.id}</span>
          <span className={`plan plan-${a.plan}`}>
            {a.plan} · {a.limit} {a.limit === 1 ? "stream" : "streams"}
          </span>
        </button>
      ))}
    </nav>
  );
}
