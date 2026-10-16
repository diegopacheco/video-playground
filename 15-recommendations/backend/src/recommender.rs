use crate::embed::{cosine, normalize, DIM};
use crate::model::{rank, Model};
use crate::types::{Profile, Scored};
use std::collections::{HashMap, HashSet};

pub trait Recommender: Send + Sync {
    fn name(&self) -> &'static str;
    fn recommend(&self, model: &Model, profile: &Profile, k: usize) -> Vec<Scored>;
}

pub struct PopularityRecommender;
pub struct CfRecommender;
pub struct ContentRecommender;
pub struct HybridRecommender;

fn watched(profile: &Profile) -> HashSet<i32> {
    profile.watched.iter().copied().collect()
}

impl Recommender for PopularityRecommender {
    fn name(&self) -> &'static str {
        "popular"
    }

    fn recommend(&self, model: &Model, profile: &Profile, k: usize) -> Vec<Scored> {
        let scores = model.titles.keys().map(|id| (*id, model.popularity.normalized(*id))).collect();
        rank(scores, &watched(profile), k)
    }
}

impl Recommender for CfRecommender {
    fn name(&self) -> &'static str {
        "cf"
    }

    fn recommend(&self, model: &Model, profile: &Profile, k: usize) -> Vec<Scored> {
        let mut scores: HashMap<i32, f32> = HashMap::new();
        for id in &profile.watched {
            for (other, sim) in model.cf.neighbors(*id) {
                *scores.entry(*other).or_insert(0.0) += sim;
            }
        }
        rank(scores, &watched(profile), k)
    }
}

pub fn profile_vector(model: &Model, profile: &Profile) -> Option<Vec<f32>> {
    let mut v = vec![0.0f32; DIM];
    let mut any = false;
    for id in &profile.watched {
        if let Some(e) = model.embedding(*id) {
            v.iter_mut().zip(e).for_each(|(a, b)| *a += b);
            any = true;
        }
    }
    if !profile.genres.is_empty() {
        let g = model.embedder.embed_genres(&profile.genres);
        v.iter_mut().zip(&g).for_each(|(a, b)| *a += b);
        any = true;
    }
    if !any {
        return None;
    }
    normalize(&mut v);
    Some(v)
}

impl Recommender for ContentRecommender {
    fn name(&self) -> &'static str {
        "content"
    }

    fn recommend(&self, model: &Model, profile: &Profile, k: usize) -> Vec<Scored> {
        let Some(query) = profile_vector(model, profile) else {
            return Vec::new();
        };
        let scores = model.embeddings.iter().map(|(id, e)| (*id, cosine(&query, e))).collect();
        rank(scores, &watched(profile), k)
    }
}

fn scaled(items: Vec<Scored>) -> HashMap<i32, f32> {
    let max = items.iter().map(|s| s.score).fold(0.0f32, f32::max);
    items
        .into_iter()
        .map(|s| (s.title_id, if max > 0.0 { s.score / max } else { 0.0 }))
        .collect()
}

pub fn hybrid_weights(watch_count: usize) -> (f32, f32, f32) {
    match watch_count {
        0 => (0.0, 0.6, 0.4),
        1..=3 => (0.4, 0.4, 0.2),
        _ => (0.6, 0.3, 0.1),
    }
}

impl Recommender for HybridRecommender {
    fn name(&self) -> &'static str {
        "hybrid"
    }

    fn recommend(&self, model: &Model, profile: &Profile, k: usize) -> Vec<Scored> {
        let all = model.titles.len();
        let (wc, wt, wp) = hybrid_weights(profile.watched.len());
        let parts = [
            (wc, scaled(CfRecommender.recommend(model, profile, all))),
            (wt, scaled(ContentRecommender.recommend(model, profile, all))),
            (wp, scaled(PopularityRecommender.recommend(model, profile, all))),
        ];
        let mut scores: HashMap<i32, f32> = HashMap::new();
        for (w, part) in parts {
            for (id, s) in part {
                *scores.entry(id).or_insert(0.0) += w * s;
            }
        }
        rank(scores, &watched(profile), k)
    }
}

pub fn by_name(name: &str) -> Option<Box<dyn Recommender>> {
    match name {
        "popular" => Some(Box::new(PopularityRecommender)),
        "cf" => Some(Box::new(CfRecommender)),
        "content" => Some(Box::new(ContentRecommender)),
        "hybrid" => Some(Box::new(HybridRecommender)),
        _ => None,
    }
}

pub fn all() -> Vec<Box<dyn Recommender>> {
    vec![Box::new(PopularityRecommender), Box::new(CfRecommender), Box::new(ContentRecommender), Box::new(HybridRecommender)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Title;

    fn title(id: i32, genres: &[&str], tags: &[&str]) -> Title {
        Title {
            id,
            name: format!("T{id}"),
            genres: genres.iter().map(|s| s.to_string()).collect(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            synopsis: String::new(),
            year: 2020,
            is_new: false,
        }
    }

    fn fixture() -> Model {
        let titles = vec![
            title(1, &["Action"], &["swordplay"]),
            title(2, &["Action"], &["swordplay", "rivalry"]),
            title(3, &["Romance"], &["cafe"]),
            title(4, &["Romance"], &["cafe", "confession"]),
            title(5, &["Comedy"], &["gag"]),
            title(6, &["Mecha"], &["giant robots"]),
        ];
        let histories = HashMap::from([
            (1, vec![1, 2]),
            (2, vec![1, 2, 5]),
            (3, vec![3, 4, 5]),
            (4, vec![3, 4, 5]),
            (5, vec![5]),
            (6, vec![5, 1]),
        ]);
        Model::build(&titles, &histories)
    }

    fn ids(items: &[Scored]) -> Vec<i32> {
        items.iter().map(|s| s.title_id).collect()
    }

    #[test]
    fn every_recommender_never_returns_what_the_user_already_watched() {
        let m = fixture();
        let p = Profile { watched: vec![1, 5], genres: vec![] };
        for r in all() {
            let got = ids(&r.recommend(&m, &p, 10));
            assert!(!got.contains(&1) && !got.contains(&5), "{} leaked a watched title", r.name());
        }
    }

    #[test]
    fn cf_follows_co_watch_behavior() {
        let m = fixture();
        let got = CfRecommender.recommend(&m, &Profile { watched: vec![3], genres: vec![] }, 1);
        assert_eq!(ids(&got), vec![4]);
    }

    #[test]
    fn cf_cannot_help_a_user_with_no_history() {
        let m = fixture();
        assert!(CfRecommender.recommend(&m, &Profile::default(), 5).is_empty());
    }

    #[test]
    fn content_serves_a_never_watched_title_that_cf_cannot_see() {
        let m = fixture();
        let p = Profile { watched: vec![], genres: vec!["Mecha".into()] };
        assert_eq!(ids(&ContentRecommender.recommend(&m, &p, 1)), vec![6]);
        assert!(!ids(&CfRecommender.recommend(&m, &p, 10)).contains(&6));
    }

    #[test]
    fn cold_start_hybrid_mixes_onboarding_genres_with_popularity() {
        let m = fixture();
        let p = Profile { watched: vec![], genres: vec!["Romance".into()] };
        let got = ids(&HybridRecommender.recommend(&m, &p, 3));
        assert!(got[..2].contains(&3) && got[..2].contains(&4), "picked genre should lead: {got:?}");
        assert!(got.contains(&5), "the most popular title should still surface: {got:?}");
    }

    #[test]
    fn hybrid_trusts_cf_more_as_history_grows() {
        assert_eq!(hybrid_weights(0).0, 0.0);
        assert!(hybrid_weights(10).0 > hybrid_weights(2).0);
    }

    #[test]
    fn unknown_algorithm_names_are_rejected() {
        assert!(by_name("hybrid").is_some());
        assert!(by_name("magic").is_none());
    }
}
