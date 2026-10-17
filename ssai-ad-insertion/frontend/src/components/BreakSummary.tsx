import type { StitchedBreak } from "../types";

interface Props {
  breaks: StitchedBreak[];
}

export function BreakSummary({ breaks }: Props) {
  return (
    <div className="breaks">
      {breaks.map((b) => (
        <div key={b.id} className="break">
          <div className="break-head">
            <strong>{b.id}</strong>
            <span>
              planned {b.planned}s · filled {b.filled}s
            </span>
          </div>
          <div className="pod">
            {b.creatives.length === 0 && <span className="chip bypass">bypass, original content kept</span>}
            {b.creatives.map((c, i) => (
              <span key={i} className={`chip c-${c}`}>
                {c}
              </span>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
