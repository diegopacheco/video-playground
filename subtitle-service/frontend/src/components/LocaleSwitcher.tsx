import type { TrackInfo } from "../types";

interface Props {
  tracks: TrackInfo[];
  value: string;
  onChange: (locale: string) => void;
}

export function LocaleSwitcher({ tracks, value, onChange }: Props) {
  return (
    <div className="locales" role="radiogroup" aria-label="Subtitle locale">
      {tracks.map((t) => (
        <button
          key={t.locale}
          role="radio"
          aria-checked={t.locale === value}
          className={t.locale === value ? "chip active" : "chip"}
          onClick={() => onChange(t.locale)}
        >
          <span className="code">{t.locale}</span> {t.name}
        </button>
      ))}
    </div>
  );
}
