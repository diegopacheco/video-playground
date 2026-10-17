import { useEffect, useState } from "react";
import { stats as fetchStats } from "./api/client";
import { QueryInfo } from "./components/QueryInfo";
import { ResultCard } from "./components/ResultCard";
import { SearchBox } from "./components/SearchBox";
import { useSearch } from "./hooks/useSearch";
import type { IndexStats } from "./types/search";

export function App() {
  const [query, setQuery] = useState("shingeki");
  const [stats, setStats] = useState<IndexStats | null>(null);
  const { response, error, loading } = useSearch(query, 10);

  useEffect(() => {
    fetchStats().then(setStats).catch(() => setStats(null));
  }, []);

  return (
    <main>
      <header>
        <h1>Catalog Search</h1>
        <p>
          Own n-gram index · trigrams for latin, bigrams for kanji/kana · Damerau-Levenshtein ranking
          {stats && ` · ${stats.documents} titles · ${stats.grams} grams · ${stats.postings} postings`}
        </p>
      </header>
      <SearchBox value={query} onChange={setQuery} />
      {error && <p className="error">{error}</p>}
      {response && <QueryInfo response={response} />}
      {response && response.results.length === 0 && !loading && <p className="empty">No titles match.</p>}
      <ol className="results">
        {response?.results.map((hit, i) => <ResultCard key={`${response.query}-${hit.id}`} hit={hit} rank={i + 1} />)}
      </ol>
    </main>
  );
}
