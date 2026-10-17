use std::collections::HashMap;

#[derive(Default)]
pub struct Popularity {
    counts: HashMap<i32, u32>,
    max: u32,
}

impl Popularity {
    pub fn fit(histories: &HashMap<i32, Vec<i32>>) -> Self {
        let mut counts: HashMap<i32, u32> = HashMap::new();
        for items in histories.values() {
            for i in items {
                *counts.entry(*i).or_insert(0) += 1;
            }
        }
        let max = counts.values().copied().max().unwrap_or(0);
        Self { counts, max }
    }

    pub fn count(&self, id: i32) -> u32 {
        self.counts.get(&id).copied().unwrap_or(0)
    }

    pub fn normalized(&self, id: i32) -> f32 {
        if self.max == 0 { 0.0 } else { self.count(id) as f32 / self.max as f32 }
    }
}
