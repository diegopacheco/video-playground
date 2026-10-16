use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CachePolicy {
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
    pub no_store: bool,
}

impl CachePolicy {
    pub fn parse(header: Option<&str>) -> Self {
        let mut policy = CachePolicy::default();
        let mut shared_max_age = None;
        for directive in header.unwrap_or("").split(',') {
            let directive = directive.trim().to_ascii_lowercase();
            let (name, value) = match directive.split_once('=') {
                Some((name, value)) => (name.trim().to_string(), value.trim().trim_matches('"').parse::<u64>().ok()),
                None => (directive.clone(), None),
            };
            match (name.as_str(), value) {
                ("no-store" | "no-cache" | "private", _) => policy.no_store = true,
                ("max-age", Some(secs)) => policy.max_age = Duration::from_secs(secs),
                ("s-maxage", Some(secs)) => shared_max_age = Some(Duration::from_secs(secs)),
                ("stale-while-revalidate", Some(secs)) => {
                    policy.stale_while_revalidate = Duration::from_secs(secs)
                }
                _ => {}
            }
        }
        if let Some(shared) = shared_max_age {
            policy.max_age = shared;
        }
        policy
    }

    pub fn cacheable(&self) -> bool {
        !self.no_store && (!self.max_age.is_zero() || !self.stale_while_revalidate.is_zero())
    }

    pub fn header_value(&self) -> String {
        if self.no_store {
            return "no-store".to_string();
        }
        format!(
            "public, max-age={}, stale-while-revalidate={}",
            self.max_age.as_secs(),
            self.stale_while_revalidate.as_secs()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_max_age_and_swr_so_the_edge_knows_how_long_to_serve() {
        let policy = CachePolicy::parse(Some("public, max-age=10, stale-while-revalidate=30"));
        assert_eq!(policy.max_age, Duration::from_secs(10));
        assert_eq!(policy.stale_while_revalidate, Duration::from_secs(30));
        assert!(policy.cacheable());
    }

    #[test]
    fn s_maxage_wins_because_the_edge_is_a_shared_cache() {
        let policy = CachePolicy::parse(Some("max-age=5, s-maxage=60"));
        assert_eq!(policy.max_age, Duration::from_secs(60));
    }

    #[test]
    fn no_store_and_private_are_never_cached_at_the_edge() {
        assert!(!CachePolicy::parse(Some("no-store, max-age=60")).cacheable());
        assert!(!CachePolicy::parse(Some("private, max-age=60")).cacheable());
    }

    #[test]
    fn missing_header_means_uncacheable_instead_of_guessing_a_ttl() {
        assert!(!CachePolicy::parse(None).cacheable());
        assert!(!CachePolicy::parse(Some("max-age=0")).cacheable());
    }

    #[test]
    fn garbage_values_are_ignored() {
        let policy = CachePolicy::parse(Some("max-age=abc, MAX-AGE=7"));
        assert_eq!(policy.max_age, Duration::from_secs(7));
    }
}
