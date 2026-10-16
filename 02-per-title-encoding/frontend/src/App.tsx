import { useEffect, useState } from "react";
import { useAnalysis } from "./hooks/useAnalysis";
import { RunPanel } from "./components/RunPanel";
import { RateQualityChart } from "./components/RateQualityChart";
import { LadderTable } from "./components/LadderTable";
import { SavingsSummary } from "./components/SavingsSummary";
import { PointsTable } from "./components/PointsTable";

const TABS = [
  { id: "curves", label: "Rate-quality curves" },
  { id: "ladders", label: "Ladders & savings" },
  { id: "grid", label: "Measurements" },
] as const;

type TabId = (typeof TABS)[number]["id"];

function tabFromHash(): TabId {
  const id = window.location.hash.replace("#", "");
  return TABS.find((t) => t.id === id)?.id ?? "curves";
}

export function App() {
  const { analysis, status, error, run } = useAnalysis();
  const [tab, setTab] = useState<TabId>(tabFromHash);
  const [metric, setMetric] = useState("vmaf");

  useEffect(() => {
    const onHash = () => setTab(tabFromHash());
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  return (
    <main>
      <header>
        <h1>Per-title encoding</h1>
        <p className="muted">
          Each title is encoded across a resolution x bitrate grid, scored with VMAF and SSIM, and its convex hull picks a custom
          bitrate ladder that is compared with one fixed generic ladder.
        </p>
        <RunPanel analysis={analysis} status={status} onRun={run} />
        {error && <div className="error">{error}</div>}
      </header>
      <nav>
        {TABS.map((t) => (
          <a key={t.id} href={`#${t.id}`} className={t.id === tab ? "active" : undefined}>
            {t.label}
          </a>
        ))}
      </nav>
      {!analysis && <p className="muted">Run the analysis to see results.</p>}
      {analysis && tab === "curves" && (
        <>
          <div className="toolbar">
            <span>Metric</span>
            {analysis.metrics.map((m) => (
              <button key={m} className={m === metric ? "chip active" : "chip"} onClick={() => setMetric(m)}>
                {m.toUpperCase()}
              </button>
            ))}
          </div>
          <div className="charts">
            {analysis.titles.map((t) => (
              <section key={t.id} className="card">
                <h2>{t.name}</h2>
                <p className="muted">{t.description}</p>
                <RateQualityChart title={t} metric={metric} targets={analysis.targets} showTargets={metric === analysis.ladder_metric} />
              </section>
            ))}
          </div>
        </>
      )}
      {analysis && tab === "ladders" && (
        <>
          <SavingsSummary titles={analysis.titles} metric={analysis.ladder_metric} />
          <div className="charts">
            {analysis.titles.map((t) => (
              <section key={t.id} className="card">
                <h2>{t.name}</h2>
                <LadderTable title={t} metric={analysis.ladder_metric} />
              </section>
            ))}
          </div>
        </>
      )}
      {analysis && tab === "grid" && (
        <div className="charts">
          {analysis.titles.map((t) => (
            <section key={t.id} className="card">
              <PointsTable title={t} metrics={analysis.metrics} ladderMetric={analysis.ladder_metric} />
            </section>
          ))}
        </div>
      )}
    </main>
  );
}
