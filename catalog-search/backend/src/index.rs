use std::collections::HashMap;

use serde::Serialize;

use crate::catalog::{Field, PreparedDoc};
use crate::ngram::index_grams;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub doc: usize,
    pub overlap: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct IndexStats {
    pub documents: usize,
    pub grams: usize,
    pub postings: usize,
}

pub trait Indexer: Send + Sync {
    fn add(&mut self, doc: usize, prepared: &PreparedDoc);
    fn candidates(&self, grams: &[String], min_overlap: f32, limit: usize) -> Vec<Candidate>;
    fn stats(&self) -> IndexStats;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Posting {
    doc: usize,
    field: Field,
}

#[derive(Debug, Default)]
pub struct NgramIndex {
    postings: HashMap<String, Vec<Posting>>,
    documents: usize,
}

impl NgramIndex {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Indexer for NgramIndex {
    fn add(&mut self, doc: usize, prepared: &PreparedDoc) {
        for field in &prepared.fields {
            for token in &field.tokens {
                for gram in index_grams(token) {
                    let list = self.postings.entry(gram).or_default();
                    let posting = Posting { doc, field: field.field };
                    if list.last() != Some(&posting) {
                        list.push(posting);
                    }
                }
            }
        }
        self.documents += 1;
    }

    fn candidates(&self, grams: &[String], min_overlap: f32, limit: usize) -> Vec<Candidate> {
        if grams.is_empty() {
            return Vec::new();
        }
        let mut hits: HashMap<Posting, u32> = HashMap::new();
        for gram in grams {
            if let Some(list) = self.postings.get(gram) {
                for posting in list {
                    *hits.entry(*posting).or_default() += 1;
                }
            }
        }
        let total = grams.len() as f32;
        let mut best: HashMap<usize, f32> = HashMap::new();
        for (posting, count) in hits {
            let overlap = count as f32 / total;
            let slot = best.entry(posting.doc).or_default();
            if overlap > *slot {
                *slot = overlap;
            }
        }
        let mut out: Vec<Candidate> = best
            .into_iter()
            .filter(|(_, overlap)| *overlap >= min_overlap)
            .map(|(doc, overlap)| Candidate { doc, overlap })
            .collect();
        out.sort_by(|a, b| b.overlap.total_cmp(&a.overlap).then(a.doc.cmp(&b.doc)));
        out.truncate(limit);
        out
    }

    fn stats(&self) -> IndexStats {
        IndexStats {
            documents: self.documents,
            grams: self.postings.len(),
            postings: self.postings.values().map(Vec::len).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Anime;

    fn doc(id: u32, romaji: &str, english: &str, japanese: &str) -> PreparedDoc {
        PreparedDoc::new(Anime {
            id,
            romaji: romaji.into(),
            english: english.into(),
            japanese: japanese.into(),
            year: 2000,
            popularity: 50,
        })
    }

    fn grams(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn overlap_is_measured_against_the_query_so_long_titles_are_not_penalised() {
        let mut index = NgramIndex::new();
        index.add(0, &doc(1, "Shingeki no Kyojin", "Attack on Titan", "進撃の巨人"));
        let found = index.candidates(&grams(&["^sh", "shi", "hin"]), 0.3, 10);
        assert_eq!(found, vec![Candidate { doc: 0, overlap: 1.0 }]);
    }

    #[test]
    fn documents_below_min_overlap_are_not_candidates() {
        let mut index = NgramIndex::new();
        index.add(0, &doc(1, "Naruto", "Naruto", "なると"));
        let found = index.candidates(&grams(&["^na", "zzz", "yyy", "xxx"]), 0.3, 10);
        assert!(found.is_empty());
    }

    #[test]
    fn same_gram_twice_in_one_field_counts_once() {
        let mut index = NgramIndex::new();
        index.add(0, &doc(1, "Dandadan", "Dan Da Dan", "だんだだん"));
        let stats = index.stats();
        let found = index.candidates(&grams(&["^da", "dan"]), 0.0, 10);
        assert_eq!(found[0].overlap, 1.0);
        assert!(stats.postings >= stats.grams);
    }
}
