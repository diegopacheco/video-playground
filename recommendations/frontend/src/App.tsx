import { useState } from "react";
import { api } from "./api/client";
import { Catalog } from "./components/Catalog";
import { Evaluation } from "./components/Evaluation";
import { ForYou } from "./components/ForYou";
import { Onboarding } from "./components/Onboarding";
import { TitleDrawer } from "./components/TitleDrawer";
import { useAsync } from "./hooks/useAsync";
import type { Algorithm, User } from "./types";

type Tab = "foryou" | "catalog" | "eval";

const TABS: [Tab, string][] = [
  ["foryou", "For You"],
  ["catalog", "Catalog"],
  ["eval", "Evaluation"],
];

function initialUser(): number {
  const fromUrl = Number(new URLSearchParams(window.location.search).get("user"));
  return Number.isInteger(fromUrl) && fromUrl > 0 ? fromUrl : 1;
}

function initialTab(): Tab {
  const t = new URLSearchParams(window.location.search).get("tab");
  return t === "catalog" || t === "eval" ? t : "foryou";
}

export function App() {
  const [tab, setTab] = useState<Tab>(initialTab);
  const [userId, setUserId] = useState<number>(initialUser);
  const [algo, setAlgo] = useState<Algorithm>("hybrid");
  const [openId, setOpenId] = useState<number | null>(null);
  const [onboarding, setOnboarding] = useState(false);
  const [version, setVersion] = useState(0);

  const genres = useAsync(() => api.genres(), []);
  const users = useAsync(() => api.users(), [version]);
  const user = useAsync(() => api.user(userId), [userId, version]);

  const bump = () => setVersion((v) => v + 1);

  const created = (u: User) => {
    setOnboarding(false);
    setUserId(u.id);
    setTab("foryou");
    bump();
  };

  const watchedIds = new Set((user.data?.watched ?? []).map((t) => t.id));

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="logo">AR</span> Anime Recs
        </div>
        <nav className="tabs">
          {TABS.map(([key, label]) => (
            <button key={key} className={tab === key ? "on" : ""} onClick={() => setTab(key)}>
              {label}
            </button>
          ))}
        </nav>
        <div className="user-pick">
          <select value={userId} onChange={(e) => setUserId(Number(e.target.value))}>
            {(users.data ?? []).map((u) => (
              <option key={u.user.id} value={u.user.id}>
                {u.user.name} · {u.user.cluster ?? "new user"} · {u.watch_count} watched
              </option>
            ))}
          </select>
          <button className="primary" onClick={() => setOnboarding(true)}>
            New user
          </button>
        </div>
      </header>
      <main>
        {onboarding && (
          <Onboarding genres={genres.data?.genres ?? []} onCreated={created} onCancel={() => setOnboarding(false)} />
        )}
        {tab === "foryou" && user.data && (
          <ForYou user={user.data} algo={algo} version={version} onAlgo={setAlgo} onOpen={setOpenId} />
        )}
        {tab === "foryou" && user.error && <p className="error">{user.error}</p>}
        {tab === "catalog" && (
          <Catalog
            genres={genres.data?.genres ?? []}
            version={version}
            onOpen={setOpenId}
            onCreated={(id) => {
              bump();
              setOpenId(id);
            }}
          />
        )}
        {tab === "eval" && <Evaluation />}
      </main>
      {openId !== null && (
        <TitleDrawer
          id={openId}
          userId={user.data ? userId : null}
          watched={watchedIds.has(openId)}
          onOpen={setOpenId}
          onClose={() => setOpenId(null)}
          onWatched={bump}
        />
      )}
    </div>
  );
}
