import type { Policy } from "../types";

interface Props {
  policy: Policy | null;
  onChange: (policy: Policy) => void;
}

const OPTIONS: { value: Policy; label: string; hint: string }[] = [
  { value: "reject_newest", label: "Reject newest", hint: "a new device is refused when the plan is full" },
  { value: "kick_oldest", label: "Kick oldest", hint: "the longest running stream is evicted" },
];

export function PolicyToggle({ policy, onChange }: Props) {
  const hint = OPTIONS.find((o) => o.value === policy)?.hint ?? "";
  return (
    <div className="policy">
      <div className="segmented" role="radiogroup" aria-label="Over-limit policy">
        {OPTIONS.map((o) => (
          <button
            key={o.value}
            role="radio"
            aria-checked={policy === o.value}
            className={policy === o.value ? "seg active" : "seg"}
            onClick={() => onChange(o.value)}
          >
            {o.label}
          </button>
        ))}
      </div>
      <span className="muted">{hint}</span>
    </div>
  );
}
