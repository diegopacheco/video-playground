import { api } from "../api/client";
import { useAsync } from "../hooks/useAsync";
import { initials, posterColors } from "../lib/display";
import { Shelf } from "./Shelf";

interface Props {
  id: number;
  userId: number | null;
  watched: boolean;
  onOpen: (id: number) => void;
  onClose: () => void;
  onWatched: () => void;
}

export function TitleDrawer({ id, userId, watched, onOpen, onClose, onWatched }: Props) {
  const { data, error } = useAsync(() => api.title(id), [id]);

  const markWatched = async () => {
    if (userId === null) return;
    await api.watch(userId, id);
    onWatched();
  };

  return (
    <div className="overlay" onClick={onClose}>
      <aside className="drawer" onClick={(e) => e.stopPropagation()}>
        <button className="close" onClick={onClose} aria-label="close">
          ×
        </button>
        {error && <p className="error">{error}</p>}
        {data && (
          <>
            <div className="drawer-head">
              <div
                className="poster big"
                style={{ background: `linear-gradient(135deg, ${posterColors(id)[0]}, ${posterColors(id)[1]})` }}
              >
                <span className="poster-initials">{initials(data.title.name)}</span>
              </div>
              <div>
                <h2>
                  {data.title.name} {data.title.is_new && <span className="badge new inline">NEW</span>}
                </h2>
                <p className="muted">
                  {data.title.year} · {data.watches} watches
                </p>
                <div className="chips small">
                  {data.title.genres.map((g) => (
                    <span key={g} className="chip on">
                      {g}
                    </span>
                  ))}
                  {data.title.tags.map((t) => (
                    <span key={t} className="chip">
                      #{t}
                    </span>
                  ))}
                </div>
                <p>{data.title.synopsis}</p>
                {userId !== null && (
                  <button className="primary" disabled={watched} onClick={markWatched}>
                    {watched ? "Watched" : "Mark as watched"}
                  </button>
                )}
              </div>
            </div>
            <Shelf
              label="Similar titles"
              hint="pgvector HNSW, cosine distance on content embeddings"
              items={data.similar_content}
              onOpen={onOpen}
            />
            <Shelf
              label="People who watched this also watched"
              hint="item-item cosine on co-watch counts"
              items={data.also_watched}
              empty="No co-watch data yet. A new title is served by content similarity only."
              onOpen={onOpen}
            />
          </>
        )}
      </aside>
    </div>
  );
}
