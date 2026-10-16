use crate::entry::Entry;
use std::collections::{BTreeMap, HashMap};

pub trait Cache: Send + 'static {
    fn get(&mut self, key: &str) -> Option<Entry>;
    fn put(&mut self, key: String, entry: Entry) -> Vec<String>;
    fn len(&self) -> usize;
    fn bytes(&self) -> usize;
    fn capacity_bytes(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct Slot {
    entry: Entry,
    tick: u64,
}

pub struct LruCache {
    capacity: usize,
    used: usize,
    tick: u64,
    slots: HashMap<String, Slot>,
    recency: BTreeMap<u64, String>,
}

impl LruCache {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            used: 0,
            tick: 0,
            slots: HashMap::new(),
            recency: BTreeMap::new(),
        }
    }

    fn next_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    fn remove(&mut self, key: &str) {
        if let Some(slot) = self.slots.remove(key) {
            self.recency.remove(&slot.tick);
            self.used -= slot.entry.size();
        }
    }

    fn evict_until_fits(&mut self, incoming: usize) -> Vec<String> {
        let mut evicted = Vec::new();
        while self.used + incoming > self.capacity {
            let Some((_, key)) = self.recency.pop_first() else {
                break;
            };
            if let Some(slot) = self.slots.remove(&key) {
                self.used -= slot.entry.size();
            }
            evicted.push(key);
        }
        evicted
    }
}

impl Cache for LruCache {
    fn get(&mut self, key: &str) -> Option<Entry> {
        let tick = self.next_tick();
        let slot = self.slots.get_mut(key)?;
        self.recency.remove(&slot.tick);
        slot.tick = tick;
        self.recency.insert(tick, key.to_string());
        Some(slot.entry.clone())
    }

    fn put(&mut self, key: String, entry: Entry) -> Vec<String> {
        let size = entry.size();
        self.remove(&key);
        if size > self.capacity {
            return Vec::new();
        }
        let evicted = self.evict_until_fits(size);
        let tick = self.next_tick();
        self.recency.insert(tick, key.clone());
        self.slots.insert(key, Slot { entry, tick });
        self.used += size;
        evicted
    }

    fn len(&self) -> usize {
        self.slots.len()
    }

    fn bytes(&self) -> usize {
        self.used
    }

    fn capacity_bytes(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_control::CachePolicy;
    use bytes::Bytes;
    use tokio::time::Instant;

    fn entry(size: usize) -> Entry {
        Entry {
            body: Bytes::from(vec![0u8; size]),
            content_type: String::new(),
            policy: CachePolicy::default(),
            stored_at: Instant::now(),
        }
    }

    #[test]
    fn evicts_least_recently_used_not_least_recently_inserted() {
        let mut cache = LruCache::new(300);
        cache.put("a".into(), entry(100));
        cache.put("b".into(), entry(100));
        cache.put("c".into(), entry(100));
        assert!(cache.get("a").is_some());
        let evicted = cache.put("d".into(), entry(100));
        assert_eq!(evicted, vec!["b".to_string()]);
        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_none());
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
    }

    #[test]
    fn bound_is_bytes_not_entry_count_so_one_big_object_pushes_out_many_small_ones() {
        let mut cache = LruCache::new(300);
        cache.put("a".into(), entry(100));
        cache.put("b".into(), entry(100));
        cache.put("c".into(), entry(100));
        let evicted = cache.put("big".into(), entry(250));
        assert_eq!(evicted, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.bytes(), 250);
        assert!(cache.bytes() <= cache.capacity_bytes());
    }

    #[test]
    fn object_larger_than_capacity_is_skipped_and_does_not_flush_the_cache() {
        let mut cache = LruCache::new(300);
        cache.put("a".into(), entry(100));
        let evicted = cache.put("huge".into(), entry(301));
        assert!(evicted.is_empty());
        assert!(cache.get("huge").is_none());
        assert!(cache.get("a").is_some());
    }

    #[test]
    fn replacing_a_key_keeps_byte_accounting_exact() {
        let mut cache = LruCache::new(300);
        cache.put("a".into(), entry(100));
        cache.put("a".into(), entry(40));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.bytes(), 40);
    }

    #[test]
    fn never_exceeds_capacity_under_churn() {
        let mut cache = LruCache::new(1000);
        for i in 0..500 {
            cache.put(format!("k{i}"), entry(37 + (i % 90)));
            assert!(cache.bytes() <= 1000);
        }
        let total: usize = (0..500)
            .filter_map(|i| cache.get(&format!("k{i}")))
            .map(|e| e.size())
            .sum();
        assert_eq!(total, cache.bytes());
    }
}
