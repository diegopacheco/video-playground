import { useEffect, useState } from "react";
import type { Manifest } from "./types/manifest";
import { fetchManifest, mediaUrl } from "./api/client";
import { HlsTab } from "./components/HlsTab";
import { DashTab } from "./components/DashTab";
import { LadderTab } from "./components/LadderTab";

type Tab = "hls" | "dash" | "ladder";

const TABS: { id: Tab; label: string }[] = [
  { id: "hls", label: "HLS player" },
  { id: "dash", label: "DASH player" },
  { id: "ladder", label: "Ladder and segments" },
];

function initialTab(): Tab {
  const hash = window.location.hash.replace("#", "");
  return TABS.some((t) => t.id === hash) ? (hash as Tab) : "hls";
}

export function App() {
  const [tab, setTab] = useState<Tab>(initialTab);
  const [manifest, setManifest] = useState<Manifest | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchManifest().then(setManifest, (e: Error) => setError(e.message));
    const onHash = () => setTab(initialTab());
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  const select = (t: Tab) => {
    window.location.hash = t;
  };

  return (
    <div className="app">
      <header>
        <h1>HLS/DASH Packager</h1>
        <p className="muted">One CMAF encode, two manifests. 4-rung H.264 ladder with a fixed GOP so every rung switches on the same frame.</p>
        <nav>
          {TABS.map((t) => (
            <button key={t.id} className={tab === t.id ? "tab active" : "tab"} onClick={() => select(t.id)}>
              {t.label}
            </button>
          ))}
        </nav>
      </header>
      {error && <p className="error">Backend unreachable: {error}</p>}
      {!manifest && !error && <p className="muted">Loading manifest...</p>}
      {manifest && tab === "hls" && <HlsTab url={mediaUrl(manifest.hls_master)} segmentSeconds={manifest.segment_seconds} />}
      {manifest && tab === "dash" && <DashTab url={mediaUrl(manifest.dash_mpd)} segmentSeconds={manifest.segment_seconds} />}
      {manifest && tab === "ladder" && <LadderTab manifest={manifest} />}
    </div>
  );
}
