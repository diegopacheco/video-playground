use std::time::Instant;

use serde::Serialize;

use crate::catalog::{Anime, PreparedDoc};
use crate::error::SearchError;
use crate::index::{IndexStats, Indexer, NgramIndex};
use crate::query::Query;
use crate::rank::{Breakdown, FieldHighlight, Ranker, WeightedRanker};

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub id: u32,
    pub romaji: String,
    pub english: String,
    pub japanese: String,
    pub year: u16,
    pub popularity: u8,
    pub score: f32,
    pub breakdown: Breakdown,
    pub highlights: Vec<FieldHighlight>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub normalized: String,
    pub grams: Vec<String>,
    pub candidates: usize,
    pub took_us: u64,
    pub results: Vec<Hit>,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    pub min_overlap: f32,
    pub candidate_limit: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            min_overlap: 0.3,
            candidate_limit: 100,
        }
    }
}

pub struct SearchEngine<I: Indexer, R: Ranker> {
    docs: Vec<PreparedDoc>,
    index: I,
    ranker: R,
    config: EngineConfig,
}

pub type DefaultEngine = SearchEngine<NgramIndex, WeightedRanker>;

impl DefaultEngine {
    pub fn with_defaults(catalog: Vec<Anime>) -> Self {
        Self::new(catalog, NgramIndex::new(), WeightedRanker::default(), EngineConfig::default())
    }
}

impl<I: Indexer, R: Ranker> SearchEngine<I, R> {
    pub fn new(catalog: Vec<Anime>, mut index: I, ranker: R, config: EngineConfig) -> Self {
        let docs: Vec<PreparedDoc> = catalog.into_iter().map(PreparedDoc::new).collect();
        for (i, doc) in docs.iter().enumerate() {
            index.add(i, doc);
        }
        Self { docs, index, ranker, config }
    }

    pub fn stats(&self) -> IndexStats {
        self.index.stats()
    }

    pub fn search(&self, raw: &str, limit: usize) -> Result<SearchResponse, SearchError> {
        let started = Instant::now();
        let query = Query::parse(raw)?;
        let candidates = self
            .index
            .candidates(&query.grams, self.config.min_overlap, self.config.candidate_limit);
        let mut results: Vec<Hit> = candidates
            .iter()
            .filter_map(|c| {
                let doc = &self.docs[c.doc];
                self.ranker.score(&query, doc, c.overlap).map(|s| Hit {
                    id: doc.anime.id,
                    romaji: doc.anime.romaji.clone(),
                    english: doc.anime.english.clone(),
                    japanese: doc.anime.japanese.clone(),
                    year: doc.anime.year,
                    popularity: doc.anime.popularity,
                    score: s.total,
                    breakdown: s.breakdown,
                    highlights: s.highlights,
                })
            })
            .collect();
        results.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
        results.truncate(limit);
        Ok(SearchResponse {
            query: query.raw.clone(),
            normalized: query.normalized(),
            grams: query.grams.clone(),
            candidates: candidates.len(),
            took_us: started.elapsed().as_micros() as u64,
            results,
        })
    }
}
