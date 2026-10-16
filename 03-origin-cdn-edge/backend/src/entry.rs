use crate::cache_control::CachePolicy;
use bytes::Bytes;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Expired,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub body: Bytes,
    pub content_type: String,
    pub policy: CachePolicy,
    pub stored_at: Instant,
}

impl Entry {
    pub fn size(&self) -> usize {
        self.body.len() + self.content_type.len()
    }

    pub fn age(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.stored_at)
    }

    pub fn freshness(&self, now: Instant) -> Freshness {
        let age = self.age(now);
        if age < self.policy.max_age {
            Freshness::Fresh
        } else if age < self.policy.max_age + self.policy.stale_while_revalidate {
            Freshness::Stale
        } else {
            Freshness::Expired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(max_age: u64, swr: u64) -> Entry {
        Entry {
            body: Bytes::from_static(b"x"),
            content_type: String::new(),
            policy: CachePolicy {
                max_age: Duration::from_secs(max_age),
                stale_while_revalidate: Duration::from_secs(swr),
                no_store: false,
            },
            stored_at: Instant::now(),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn freshness_moves_fresh_then_stale_then_expired() {
        let e = entry(10, 20);
        assert_eq!(e.freshness(Instant::now()), Freshness::Fresh);
        tokio::time::advance(Duration::from_secs(10)).await;
        assert_eq!(e.freshness(Instant::now()), Freshness::Stale);
        tokio::time::advance(Duration::from_secs(20)).await;
        assert_eq!(e.freshness(Instant::now()), Freshness::Expired);
    }
}
