use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct ItemCf {
    neighbors: HashMap<i32, Vec<(i32, f32)>>,
}

impl ItemCf {
    pub fn fit(histories: &HashMap<i32, Vec<i32>>) -> Self {
        let mut counts: HashMap<i32, u32> = HashMap::new();
        let mut co: HashMap<(i32, i32), u32> = HashMap::new();
        for items in histories.values() {
            let unique: Vec<i32> = items.iter().copied().collect::<HashSet<_>>().into_iter().collect();
            for (n, i) in unique.iter().enumerate() {
                *counts.entry(*i).or_insert(0) += 1;
                for j in &unique[n + 1..] {
                    let key = if i < j { (*i, *j) } else { (*j, *i) };
                    *co.entry(key).or_insert(0) += 1;
                }
            }
        }
        let mut neighbors: HashMap<i32, Vec<(i32, f32)>> = HashMap::new();
        for ((i, j), c) in co {
            let sim = c as f32 / ((counts[&i] * counts[&j]) as f32).sqrt();
            neighbors.entry(i).or_default().push((j, sim));
            neighbors.entry(j).or_default().push((i, sim));
        }
        for list in neighbors.values_mut() {
            list.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        }
        Self { neighbors }
    }

    pub fn neighbors(&self, id: i32) -> &[(i32, f32)] {
        self.neighbors.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn similarity(&self, a: i32, b: i32) -> f32 {
        self.neighbors(a).iter().find(|(id, _)| *id == b).map(|(_, s)| *s).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histories(rows: &[(i32, &[i32])]) -> HashMap<i32, Vec<i32>> {
        rows.iter().map(|(u, items)| (*u, items.to_vec())).collect()
    }

    #[test]
    fn items_always_watched_together_have_cosine_one() {
        let cf = ItemCf::fit(&histories(&[(1, &[10, 20]), (2, &[10, 20]), (3, &[30])]));
        assert!((cf.similarity(10, 20) - 1.0).abs() < 1e-6);
        assert_eq!(cf.similarity(10, 30), 0.0);
    }

    #[test]
    fn cosine_penalizes_blockbusters_that_co_occur_by_chance() {
        let cf = ItemCf::fit(&histories(&[
            (1, &[1, 2, 99]),
            (2, &[1, 2, 99]),
            (3, &[99, 5]),
            (4, &[99, 6]),
            (5, &[99, 7]),
            (6, &[99, 8]),
        ]));
        assert!(cf.similarity(1, 2) > cf.similarity(1, 99));
        assert_eq!(cf.neighbors(1)[0].0, 2);
    }

    #[test]
    fn similarity_is_symmetric() {
        let cf = ItemCf::fit(&histories(&[(1, &[1, 2, 3]), (2, &[1, 3]), (3, &[2])]));
        assert_eq!(cf.similarity(1, 3), cf.similarity(3, 1));
    }

    #[test]
    fn a_brand_new_title_has_no_neighbors() {
        let cf = ItemCf::fit(&histories(&[(1, &[1, 2])]));
        assert!(cf.neighbors(500).is_empty());
    }
}
