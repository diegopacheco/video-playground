import { useState } from "react";
import { BurnedPanel } from "./components/BurnedPanel";
import { JassubPanel } from "./components/JassubPanel";
import { LocaleSwitcher } from "./components/LocaleSwitcher";
import { NativePanel } from "./components/NativePanel";
import { PlaybackBar } from "./components/PlaybackBar";
import { ReportView } from "./components/ReportView";
import { useSyncedPlayback } from "./hooks/useSyncedPlayback";
import { useTracks } from "./hooks/useTracks";
import { MODES, type Mode } from "./types";

type View = "compare" | "report";

const params = new URLSearchParams(window.location.search);
const solo = MODES.find((m) => m === params.get("solo")) ?? null;

export function App() {
  const { tracks, error } = useTracks();
  const [locale, setLocale] = useState(params.get("locale") ?? "en-US");
  const [view, setView] = useState<View>(params.get("view") === "report" ? "report" : "compare");
  const playback = useSyncedPlayback();
  const track = tracks.find((t) => t.locale === locale) ?? tracks[0];
  const visible: Mode[] = solo ? [solo] : MODES;

  return (
    <main>
      <header className="top">
        <h1>Subtitle service</h1>
        <nav>
          <button className={view === "compare" ? "tab active" : "tab"} onClick={() => setView("compare")}>
            Compare
          </button>
          <button className={view === "report" ? "tab active" : "tab"} onClick={() => setView("report")}>
            Report
          </button>
        </nav>
      </header>
      {error && <p className="error">{error}</p>}
      {track && view === "compare" && (
        <>
          <LocaleSwitcher tracks={tracks} value={track.locale} onChange={setLocale} />
          <PlaybackBar playback={playback} />
          <div className={`grid cols-${visible.length}`}>
            {visible.map((mode, i) => {
              const ref = playback.register(i);
              if (mode === "native") return <NativePanel key={mode} track={track} videoRef={ref} />;
              if (mode === "jassub") return <JassubPanel key={mode} track={track} videoRef={ref} />;
              return <BurnedPanel key={mode} track={track} videoRef={ref} />;
            })}
          </div>
        </>
      )}
      {view === "report" && <ReportView tracks={tracks} />}
    </main>
  );
}
