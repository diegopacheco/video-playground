const SUGGESTED = ["shingeki", "進撃", "atack on titan", "鬼滅", "kimet", "なると", "ansatsu kyōshitsu", "pokemon"];

interface Props {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBox({ value, onChange }: Props) {
  return (
    <div className="search">
      <input
        autoFocus
        type="search"
        placeholder="Search in romaji, 日本語 or English"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        aria-label="search catalog"
      />
      <div className="chips">
        {SUGGESTED.map((s) => (
          <button key={s} className={s === value ? "chip active" : "chip"} onClick={() => onChange(s)}>
            {s}
          </button>
        ))}
      </div>
    </div>
  );
}
