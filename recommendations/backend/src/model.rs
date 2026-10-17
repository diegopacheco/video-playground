use crate::cf::ItemCf;
use crate::embed::{Doc, Embedder};
use crate::popularity::Popularity;
use crate::types::{Scored, Title};
use std::collections::{HashMap, HashSet};

pub struct Model {
    pub titles: HashMap<i32, Title>,
    pub embeddings: HashMap<i32, Vec<f32>>,
    pub embedder: Embedder,
    pub cf: ItemCf,
    pub popularity: Popularity,
}

fn doc(t: &Title) -> Doc<'_> {
    Doc { genres: &t.genres, tags: &t.tags, synopsis: &t.synopsis }
}

impl Model {
    pub fn build(titles: &[Title], histories: &HashMap<i32, Vec<i32>>) -> Self {
        let docs: Vec<Doc> = titles.iter().map(doc).collect();
        let embedder = Embedder::fit(&docs);
        let embeddings = titles.iter().map(|t| (t.id, embedder.embed(&doc(t)))).collect();
        Self {
            titles: titles.iter().map(|t| (t.id, t.clone())).collect(),
            embeddings,
            embedder,
            cf: ItemCf::fit(histories),
            popularity: Popularity::fit(histories),
        }
    }

    pub fn embedding(&self, id: i32) -> Option<&[f32]> {
        self.embeddings.get(&id).map(|v| v.as_slice())
    }
}

pub fn rank(scores: HashMap<i32, f32>, exclude: &HashSet<i32>, k: usize) -> Vec<Scored> {
    let mut out: Vec<Scored> = scores
        .into_iter()
        .filter(|(id, s)| !exclude.contains(id) && *s > 0.0)
        .map(|(title_id, score)| Scored { title_id, score })
        .collect();
    out.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.title_id.cmp(&b.title_id)));
    out.truncate(k);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_drops_already_watched_titles_and_breaks_ties_by_id() {
        let scores = HashMap::from([(1, 0.5), (2, 0.9), (3, 0.5), (4, 0.0)]);
        let ranked = rank(scores, &HashSet::from([2]), 10);
        let ids: Vec<i32> = ranked.iter().map(|s| s.title_id).collect();
        assert_eq!(ids, vec![1, 3]);
    }
}
