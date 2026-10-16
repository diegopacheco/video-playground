interface Props {
  genres: string[];
  selected: string[];
  onChange: (next: string[]) => void;
}

export function GenrePicker({ genres, selected, onChange }: Props) {
  const toggle = (g: string) =>
    onChange(selected.includes(g) ? selected.filter((x) => x !== g) : [...selected, g]);
  return (
    <div className="chips">
      {genres.map((g) => (
        <button
          key={g}
          type="button"
          className={selected.includes(g) ? "chip on" : "chip"}
          onClick={() => toggle(g)}
        >
          {g}
        </button>
      ))}
    </div>
  );
}
