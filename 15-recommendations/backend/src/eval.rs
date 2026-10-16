use crate::model::Model;
use crate::recommender::{all, HybridRecommender, PopularityRecommender, Recommender};
use crate::types::{Profile, Title};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub const K: usize = 10;
pub const HOLDOUT: f64 = 0.2;

#[derive(Debug, Clone, Serialize)]
pub struct EvalRow {
    pub scenario: String,
    pub algorithm: String,
    pub precision_at_10: f64,
    pub recall_at_10: f64,
    pub users: usize,
}

pub struct Split {
    pub train: HashMap<i32, Vec<i32>>,
    pub test: HashMap<i32, HashSet<i32>>,
}

pub fn split(histories: &HashMap<i32, Vec<i32>>, ratio: f64) -> Split {
    let mut train = HashMap::new();
    let mut test = HashMap::new();
    for (user, items) in histories {
        let held = ((items.len() as f64) * ratio).round() as usize;
        if items.len() < 5 || held == 0 {
            train.insert(*user, items.clone());
            continue;
        }
        let cut = items.len() - held;
        train.insert(*user, items[..cut].to_vec());
        test.insert(*user, items[cut..].iter().copied().collect());
    }
    Split { train, test }
}

fn score(
    model: &Model,
    recommender: &dyn Recommender,
    profiles: &[(Profile, &HashSet<i32>)],
    scenario: &str,
) -> EvalRow {
    let mut precision = 0.0;
    let mut recall = 0.0;
    for (profile, truth) in profiles {
        let hits = recommender.recommend(model, profile, K).iter().filter(|s| truth.contains(&s.title_id)).count() as f64;
        precision += hits / K as f64;
        recall += hits / truth.len() as f64;
    }
    let n = profiles.len().max(1) as f64;
    EvalRow {
        scenario: scenario.to_string(),
        algorithm: recommender.name().to_string(),
        precision_at_10: precision / n,
        recall_at_10: recall / n,
        users: profiles.len(),
    }
}

pub fn evaluate(titles: &[Title], histories: &HashMap<i32, Vec<i32>>, user_genres: &HashMap<i32, Vec<String>>) -> Vec<EvalRow> {
    let s = split(histories, HOLDOUT);
    let model = Model::build(titles, &s.train);
    let mut users: Vec<i32> = s.test.keys().copied().collect();
    users.sort();
    let warm: Vec<(Profile, &HashSet<i32>)> = users
        .iter()
        .map(|u| (Profile { watched: s.train[u].clone(), genres: Vec::new() }, &s.test[u]))
        .collect();
    let mut rows: Vec<EvalRow> = all().iter().map(|r| score(&model, r.as_ref(), &warm, "held-out watches")).collect();

    let cold: Vec<(Profile, &HashSet<i32>)> = users
        .iter()
        .filter_map(|u| user_genres.get(u).map(|g| (Profile { watched: Vec::new(), genres: g.clone() }, &s.test[u])))
        .collect();
    rows.push(score(&model, &PopularityRecommender, &cold, "cold-start user"));
    rows.push(score(&model, &HybridRecommender, &cold, "cold-start user"));
    rows
}

pub fn markdown(rows: &[EvalRow]) -> String {
    let mut out = String::from("| Scenario | Algorithm | Precision@10 | Recall@10 | Users |\n|---|---|---|---|---|\n");
    for r in rows {
        out.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {} |\n",
            r.scenario, r.algorithm, r.precision_at_10, r.recall_at_10, r.users
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn row<'a>(rows: &'a [EvalRow], scenario: &str, algorithm: &str) -> &'a EvalRow {
        rows.iter().find(|r| r.scenario == scenario && r.algorithm == algorithm).expect("row present")
    }

    #[test]
    fn split_never_leaks_held_out_watches_into_training() {
        let h = HashMap::from([(1, (1..=10).collect::<Vec<i32>>())]);
        let s = split(&h, 0.2);
        assert_eq!(s.test[&1], HashSet::from([9, 10]));
        assert!(s.train[&1].iter().all(|i| !s.test[&1].contains(i)));
    }

    #[test]
    fn personalized_recommenders_beat_plain_popularity_on_taste_clusters() {
        let d = seed::indexed();
        let rows = evaluate(&d.titles, &d.histories, &d.user_genres);
        let popular = row(&rows, "held-out watches", "popular").precision_at_10;
        for algo in ["cf", "content", "hybrid"] {
            let p = row(&rows, "held-out watches", algo).precision_at_10;
            assert!(p > popular, "{algo} {p} should beat popularity {popular}");
        }
    }

    #[test]
    fn onboarding_genres_beat_popularity_for_cold_start_users() {
        let d = seed::indexed();
        let rows = evaluate(&d.titles, &d.histories, &d.user_genres);
        let popular = row(&rows, "cold-start user", "popular").precision_at_10;
        let hybrid = row(&rows, "cold-start user", "hybrid").precision_at_10;
        assert!(hybrid > popular, "hybrid {hybrid} vs popular {popular}");
    }
}
