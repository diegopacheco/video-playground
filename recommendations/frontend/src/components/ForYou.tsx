import { api } from "../api/client";
import { useAsync } from "../hooks/useAsync";
import { rowHint, visibleRows } from "../lib/display";
import type { Algorithm, UserDetail } from "../types";
import { Shelf } from "./Shelf";
import { TitleCard } from "./TitleCard";

const ALGORITHMS: Algorithm[] = ["hybrid", "cf", "content", "popular"];

interface Props {
  user: UserDetail;
  algo: Algorithm;
  version: number;
  onAlgo: (a: Algorithm) => void;
  onOpen: (id: number) => void;
}

export function ForYou({ user, algo, version, onAlgo, onOpen }: Props) {
  const { data, error, loading } = useAsync(() => api.recommendations(user.user.id, algo), [user.user.id, algo, version]);

  return (
    <div>
      <div className="profile">
        <div>
          <h2>{user.user.name}</h2>
          <p className="muted">
            {user.user.cluster ? `Taste cluster: ${user.user.cluster}` : "Created in the UI"}
            {user.user.picked_genres.length > 0 && ` · Onboarding picks: ${user.user.picked_genres.join(", ")}`}
            {` · ${user.watched.length} watched`}
            {data?.cold_start && <span className="badge cold inline">COLD START</span>}
          </p>
        </div>
        <div className="segmented">
          {ALGORITHMS.map((a) => (
            <button key={a} className={a === algo ? "on" : ""} onClick={() => onAlgo(a)}>
              {a}
            </button>
          ))}
        </div>
      </div>
      {error && <p className="error">{error}</p>}
      {loading && !data && <p className="muted">Loading recommendations...</p>}
      {data &&
        visibleRows(data.rows).map((row) => (
          <Shelf key={row.kind} label={row.label} hint={rowHint(row.kind)} items={row.items} onOpen={onOpen} />
        ))}
      <section className="shelf">
        <div className="shelf-head">
          <h3>Watch history</h3>
          <span className="shelf-hint">newest first</span>
        </div>
        {user.watched.length === 0 ? (
          <p className="muted">No watches yet. Open any title and mark it as watched.</p>
        ) : (
          <div className="shelf-row">
            {user.watched.map((t) => (
              <TitleCard key={t.id} title={t} onOpen={onOpen} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
