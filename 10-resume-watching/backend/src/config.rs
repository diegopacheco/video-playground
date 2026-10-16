use crate::error::StoreError;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub store: String,
    pub redis_url: String,
    pub scylla_node: String,
    pub flush_secs: u64,
}

fn var(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn parse<T: std::str::FromStr>(name: &str, default: &str) -> Result<T, StoreError> {
    let raw = var(name, default);
    raw.parse().map_err(|_| StoreError::Config(format!("{name}={raw} is not valid")))
}

impl Config {
    pub fn from_env() -> Result<Self, StoreError> {
        let config = Self {
            port: parse("PORT", "8110")?,
            store: var("PLAYHEAD_STORE", "redis"),
            redis_url: var("REDIS_URL", "redis://127.0.0.1:6110"),
            scylla_node: var("SCYLLA_NODE", "127.0.0.1:6210"),
            flush_secs: parse("FLUSH_SECS", "15")?,
        };
        if config.flush_secs == 0 {
            return Err(StoreError::Config("FLUSH_SECS must be at least 1".into()));
        }
        Ok(config)
    }
}
