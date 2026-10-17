use crate::entitlement::Status;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub status: Status,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

type Key = (String, String);

#[derive(Default)]
pub struct ResponseCache {
    entries: RwLock<HashMap<Key, Vec<Entry>>>,
}

impl ResponseCache {
    pub fn get(&self, episode: &str, country: &str, now: DateTime<Utc>) -> Option<Status> {
        let entries = self.entries.read().unwrap_or_else(PoisonError::into_inner);
        entries
            .get(&(episode.to_string(), country.to_string()))?
            .iter()
            .find(|e| e.valid_from <= now && now < e.expires_at)
            .map(|e| e.status)
    }

    pub fn put(&self, episode: &str, country: &str, entry: Entry, now: DateTime<Utc>) {
        let mut entries = self.entries.write().unwrap_or_else(PoisonError::into_inner);
        let slot = entries
            .entry((episode.to_string(), country.to_string()))
            .or_default();
        slot.retain(|e| e.expires_at > now && e.valid_from != entry.valid_from);
        slot.push(entry);
    }

    pub fn invalidate_episode(&self, episode: &str) {
        let mut entries = self.entries.write().unwrap_or_else(PoisonError::into_inner);
        entries.retain(|(e, _), _| e != episode);
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap_or_else(PoisonError::into_inner);
        entries.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
